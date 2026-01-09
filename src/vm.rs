use anyhow::Error;
use log::debug;
use std::rc::Rc;

use crate::alias::{StoredChunk, StoredObject, StoredValue, VoidResult};
use crate::bin_op::BinOpKind;
use crate::chunk::OpCodeKind;
use crate::errors::RuntimeError;
use crate::errors::RuntimeErrorKind;
use crate::namespace::NameSpace;
use crate::object::function::FunctionObject;
use crate::value::{Compare, Value};
use crate::{calc, rc_refcell};

type ValueStack = Vec<StoredValue>;
const FRAMES_MAX: usize = 64;

pub struct VirtualMachine<'a> {
    value_stack: ValueStack,
    globals: &'a mut NameSpace,
    frame_count: usize,
    frames: Vec<CallFrame>,
}

pub struct CallFrame {
    function: StoredObject<FunctionObject>,
    ip: usize,
    slot_start: usize,
}

impl CallFrame {
    pub fn new(function: StoredObject<FunctionObject>, ip: usize, slot_start: usize) -> Self {
        Self {
            function,
            ip,
            slot_start,
        }
    }
}

impl<'a> VirtualMachine<'a> {
    pub fn new(globals: &'a mut NameSpace) -> Self {
        Self {
            value_stack: vec![],
            globals,
            frames: Vec::with_capacity(FRAMES_MAX),
            frame_count: 0,
        }
    }

    pub fn reset_ip(&mut self) {
        self.frames[0].ip = 0;
    }

    pub fn exec(&mut self) -> VoidResult {
        debug!("Executing chunk:");
        debug!("\n{}", self.current_chunk().borrow());
        debug!(
            "Chunk constants: {:?}",
            self.current_chunk().borrow().constants
        );

        loop {
            let kind = {
                let current_chunk = self.current_chunk();
                let bchunk = current_chunk.borrow();
                let Some(instruction) = bchunk.get(*self.ip()) else {
                    return Ok(());
                };
                instruction.kind().clone()
            };

            debug!("Executing opcode: {kind}");
            debug!("Value stack: {:?}", self.value_stack);

            match kind {
                OpCodeKind::Const { const_idx } => {
                    self.op_const(const_idx);
                }
                OpCodeKind::Negate => self.op_negate()?,
                OpCodeKind::Add => self.bin_op(BinOpKind::Add)?,
                OpCodeKind::Sub => self.bin_op(BinOpKind::Sub)?,
                OpCodeKind::Mul => self.bin_op(BinOpKind::Mul)?,
                OpCodeKind::Div => self.bin_op(BinOpKind::Div)?,
                OpCodeKind::Null => {
                    self.push_value(Value::Null);
                }
                OpCodeKind::True => {
                    self.push_value(Value::Boolean(true));
                }
                OpCodeKind::False => {
                    self.push_value(Value::Boolean(false));
                }
                OpCodeKind::Not => {
                    let value = self.pop_or_err()?;
                    self.push_value(Value::Boolean(!value.borrow().as_bool()));
                }
                OpCodeKind::Eq => self.op_cmp(Compare::Equal)?,
                OpCodeKind::Gt => self.op_cmp(Compare::Greater)?,
                OpCodeKind::Lt => self.op_cmp(Compare::Lower)?,
                OpCodeKind::Print => self.op_print()?,
                OpCodeKind::Pop => {
                    self.pop_or_err()?;
                }
                OpCodeKind::DefineGlobal { name_idx } => self.op_define_global(name_idx)?,
                OpCodeKind::ReadGlobal { name_idx } => self.op_read_global(name_idx)?,
                OpCodeKind::SetGlobal { name_idx } => self.op_set_global(name_idx)?,
                OpCodeKind::ReadLocal { name_idx } => self.op_read_local(name_idx)?,
                OpCodeKind::SetLocal { name_idx } => self.op_set_local(name_idx)?,
                OpCodeKind::JumpIfFalse { offset } => self.op_jump_if_false(offset)?,
                OpCodeKind::Jump { offset } => self.op_jump(offset),
                OpCodeKind::Loop { offset } => self.op_loop(offset),
            }

            if !matches!(kind, OpCodeKind::Loop { .. }) {
                self.increment_ip();
            }
        }
    }

    pub fn add_frame(&mut self, frame: CallFrame) {
        debug!("Added frame\n{}", frame.function.borrow().chunk.borrow());
        self.frames.push(frame);
        self.frame_count = 1;
    }

    fn current_frame(&self) -> &CallFrame {
        &self.frames[self.frame_count - 1]
    }

    fn current_frame_mut(&mut self) -> &mut CallFrame {
        &mut self.frames[self.frame_count - 1]
    }

    fn current_chunk(&self) -> StoredChunk {
        self.current_frame().function.borrow().chunk.clone()
    }

    fn ip(&self) -> &usize {
        &self.current_frame().ip
    }

    fn add_ip(&mut self, offset: usize) {
        self.current_frame_mut().ip += offset;
    }

    fn sub_ip(&mut self, offset: usize) {
        self.frames[0].ip -= offset;
    }

    fn increment_ip(&mut self) {
        self.add_ip(1);
    }

    fn stack_index(&self, name_idx: usize) -> usize {
        self.current_frame().slot_start + name_idx
    }

    fn runtime_error(&mut self, kind: RuntimeErrorKind) -> Error {
        if self.ip() == &0 {
            return RuntimeError { kind, line: 0 }.into();
        };
        let current_chunk = self.current_chunk();
        let bchunk = current_chunk.borrow();
        let Some(prev_instruction) = bchunk.get(self.ip() - 1) else {
            panic!("Cannot get previous instruction");
        };

        RuntimeError {
            kind,
            line: prev_instruction.line(),
        }
        .into()
    }

    fn peek(&self) -> Result<StoredValue, Error> {
        let Some(value) = self.value_stack.last().cloned() else {
            panic!("Missing stack value in peek()!");
        };
        Ok(value)
    }

    fn push_value(&mut self, value: Value) {
        self.value_stack.push(rc_refcell!(value));
    }

    pub fn push_stored_value(&mut self, value: StoredValue) {
        self.value_stack.push(value);
    }

    fn pop_or_err(&mut self) -> Result<StoredValue, Error> {
        let Some(value) = self.value_stack.pop() else {
            panic!("Missing stack value in pop()!");
        };
        Ok(value)
    }

    fn as_vm_result<T>(&mut self, result: Result<T, RuntimeErrorKind>) -> Result<T, Error> {
        if let Err(error) = result {
            return Err(self.runtime_error(error));
        }
        Ok(result.unwrap())
    }

    fn bin_op(&mut self, kind: BinOpKind) -> VoidResult {
        let b = self.pop_or_err()?;
        let a = self.pop_or_err()?;

        match (&*a.borrow(), &*b.borrow()) {
            (Value::Float(a_val), Value::Float(b_val)) => {
                let calculated = calc!(a_val, b_val, kind.to_string().as_str());
                self.push_value(Value::Float(calculated));
            }
            (Value::Object(a), Value::Object(b)) => {
                let result = self.as_vm_result(a.borrow().add(b))?;
                self.push_stored_value(result);
            }
            (val1, val2) => {
                return Err(self.runtime_error(RuntimeErrorKind::OperationNotSupported {
                    op: kind.to_string(),
                    target: format!("between {} and {}", val1.type_name(), val2.type_name()),
                }));
            }
        }

        Ok(())
    }

    fn read_identifier_const(&self, idx: usize) -> Rc<String> {
        let current_chunk = self.current_chunk();
        let bchunk = current_chunk.borrow();
        let const_value = bchunk.get_const(idx).unwrap();

        match &*const_value.borrow() {
            Value::Identifier(identifier) => identifier.clone(),
            _ => unreachable!(),
        }
    }

    fn op_const(&mut self, const_idx: usize) {
        let cloned_value = {
            let current_chunk = self.current_chunk();
            let bchunk = current_chunk.borrow();
            let const_value = bchunk
                .get_const(const_idx)
                .unwrap_or_else(|| panic!("Missing value with index {const_idx} in chunk!"));
            debug!("Pushed const: {}", const_value.borrow());
            const_value.clone()
        };
        self.push_stored_value(cloned_value);
    }

    fn op_negate(&mut self) -> VoidResult {
        let peek = self.peek()?;
        if !peek.borrow().support_negation() {
            return Err(self.runtime_error(RuntimeErrorKind::OperationNotSupported {
                op: "-".to_owned(),
                target: format!("for {}", peek.borrow()),
            }));
        }

        let value = self.pop_or_err()?;
        match &*value.borrow() {
            Value::Float(float_value) => {
                self.push_value(Value::Float(-float_value));
            }
            _ => unreachable!(),
        };
        Ok(())
    }

    fn op_cmp(&mut self, expected: Compare) -> VoidResult {
        let b = self.pop_or_err()?;
        let a = self.pop_or_err()?;

        let cmp_result = a.borrow().cmp(&b.borrow());
        if cmp_result.is_err() {
            #[allow(clippy::unnecessary_unwrap)]
            return Err(self.runtime_error(cmp_result.unwrap_err()));
        }
        let result = cmp_result.unwrap() == expected;
        self.push_value(Value::Boolean(result));
        Ok(())
    }

    fn op_print(&mut self) -> VoidResult {
        let value = self.pop_or_err()?;
        debug!("Popped value: {value:?}");
        println!("{}", value.borrow());
        Ok(())
    }

    fn op_define_global(&mut self, name_idx: usize) -> VoidResult {
        let name = self.read_identifier_const(name_idx);

        if self.globals.get(&name).is_some() {
            return Err(
                self.runtime_error(RuntimeErrorKind::AlreadyDefinedVariable {
                    name: name.to_string(),
                }),
            );
        }
        self.globals.insert(name, self.peek()?);
        self.pop_or_err()?;
        Ok(())
    }

    fn op_read_global(&mut self, name_idx: usize) -> VoidResult {
        let name = self.read_identifier_const(name_idx);
        let Some(value) = self.globals.get(&name) else {
            return Err(self.runtime_error(RuntimeErrorKind::UndefinedVariable {
                name: name.to_string(),
            }));
        };
        self.push_stored_value(value);
        Ok(())
    }

    fn op_set_global(&mut self, name_idx: usize) -> VoidResult {
        let name = self.read_identifier_const(name_idx);

        let Some(_) = self.globals.get(&name) else {
            return Err(self.runtime_error(RuntimeErrorKind::UndefinedVariable {
                name: name.to_string(),
            }));
        };

        self.globals.insert(name, self.peek()?);
        Ok(())
    }

    fn op_read_local(&mut self, name_idx: usize) -> VoidResult {
        let Some(value) = self.value_stack.get(self.stack_index(name_idx)) else {
            panic!("Missing stack value in read local!");
        };
        let cloned_value = value.clone();
        self.push_stored_value(cloned_value);
        Ok(())
    }

    fn op_set_local(&mut self, name_idx: usize) -> VoidResult {
        let idx = self.stack_index(name_idx);
        self.value_stack[idx] = self.peek()?;
        Ok(())
    }

    fn op_jump_if_false(&mut self, offset: usize) -> VoidResult {
        if !self.peek()?.borrow().as_bool() {
            self.add_ip(offset);
        }
        Ok(())
    }

    fn op_jump(&mut self, offset: usize) {
        self.add_ip(offset);
    }

    fn op_loop(&mut self, offset: usize) {
        self.sub_ip(offset);
    }
}
