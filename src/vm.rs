use std::rc::Rc;

use anyhow::Error;

use crate::alias::{StoredChunk, StoredNameSpace, StoredValue, VoidResult};
use crate::bin_op::BinOpKind;
use crate::chunk::OpCodeKind;
use crate::errors::RuntimeErrorKind;
use crate::errors::RuntimeError;
use crate::namespace::NameSpace;
use crate::rc_refcell;
use crate::value::{Compare, Value};

type ValueStack = Vec<StoredValue>;

pub struct VirtualMachine {
    chunk: StoredChunk,
    ip: usize, // instruction pointer
    debug_trace: bool,
    value_stack: ValueStack,
    globals: StoredNameSpace,
}

macro_rules! calc {
    ($a:expr, $b:expr, $op:expr) => {{
        match $op {
            "+" => $a + $b,
            "-" => $a - $b,
            "*" => $a * $b,
            "/" => $a / $b,
            _ => panic!("Unsupported operator: {}", $op),
        }
    }};
}

impl VirtualMachine {
    pub fn new(chunk: StoredChunk, debug_trace: bool) -> Self {
        Self {
            chunk,
            ip: 0,
            debug_trace,
            value_stack: vec![],
            globals: rc_refcell!(NameSpace::new()),
        }
    }

    pub fn exec(&mut self) -> VoidResult {
        if self.debug_trace {
            println!("Executing this chunk:");
            println!("{}", self.chunk.borrow());
            println!()
        }
        loop {
            let kind = {
                let bchunk = self.chunk.borrow();
                let Some(instruction) = bchunk.get(self.ip) else {
                    return Ok(());
                };
                if self.debug_trace {
                    println!("{instruction}");
                }
                instruction.kind().clone()
            };

            if self.debug_trace {
                println!("{kind}");
            }

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
            }
            self.ip += 1;
        }
    }

    fn runtime_error(&self, kind: RuntimeErrorKind) -> Error {
        let bchunk = self.chunk.borrow();
        let Some(prev_instruction) = bchunk.get(self.ip - 1) else {
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
            return Err(self.runtime_error(RuntimeErrorKind::MissingValue));
        };
        Ok(value)
    }

    fn push_value(&mut self, value: Value) {
        self.value_stack.push(rc_refcell!(value));
    }

    fn push_stored_value(&mut self, value: StoredValue) {
        self.value_stack.push(value);
    }

    fn pop_or_err(&mut self) -> Result<StoredValue, Error> {
        let Some(value) = self.value_stack.pop() else {
            return Err(self.runtime_error(RuntimeErrorKind::MissingValue));
        };
        Ok(value)
    }

    fn as_vm_result<T>(&self, result: Result<T, RuntimeErrorKind>) -> Result<T, Error> {
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
                let result = self.as_vm_result(a.add(b))?;
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
        let bchunk = self.chunk.borrow();
        let const_value = bchunk.get_const(idx).unwrap();

        match &*const_value.borrow() {
            Value::Identifier(identifier) => identifier.clone(),
            _ => unreachable!(),
        }
    }

    fn op_const(&mut self, const_idx: usize) {
        let cloned_value = {
            let bchunk = self.chunk.borrow();
            let const_value = bchunk.get_const(const_idx).unwrap();
            if self.debug_trace {
                println!("Pushed const: {}", const_value.borrow());
            }
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
        println!("{}", value.borrow());
        Ok(())
    }

    fn op_define_global(&mut self, name_idx: usize) -> VoidResult {
        let name = self.read_identifier_const(name_idx);

        if self.globals.borrow_mut().get(&name).is_some() {
            return Err(
                self.runtime_error(RuntimeErrorKind::AlreadyDefinedVariable {
                    name: name.to_string(),
                }),
            );
        }
        self.globals.borrow_mut().insert(name, self.peek()?);
        self.pop_or_err()?;
        Ok(())
    }

    fn op_read_global(&mut self, name_idx: usize) -> VoidResult {
        let name = self.read_identifier_const(name_idx);
        let Some(value) = self.globals.borrow().get(&name) else {
            return Err(self.runtime_error(RuntimeErrorKind::UndefinedVariable {
                name: name.to_string(),
            }));
        };
        self.push_stored_value(value);
        Ok(())
    }

    fn op_set_global(&mut self, name_idx: usize) -> VoidResult {
        let name = self.read_identifier_const(name_idx);

        let Some(_) = self.globals.borrow().get(&name) else {
            return Err(self.runtime_error(RuntimeErrorKind::UndefinedVariable {
                name: name.to_string(),
            }));
        };

        self.globals.borrow_mut().insert(name, self.peek()?);
        Ok(())
    }

    fn op_read_local(&mut self, name_idx: usize) -> VoidResult {
        let Some(value) = self.value_stack.get(name_idx) else {
            return Err(self.runtime_error(RuntimeErrorKind::MissingValue));
        };
        let cloned_value = value.clone();
        self.push_stored_value(cloned_value);
        Ok(())
    }

    fn op_set_local(&mut self, name_idx: usize) -> VoidResult {
        self.value_stack[name_idx] = self.peek()?;
        Ok(())
    }
}
