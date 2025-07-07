use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Error;

use crate::alias::{StoredChunk, StoredNameSpace, StoredValue, VoidResult};
use crate::bin_op::BinOpKind;
use crate::chunk::OpCodeKind;
use crate::errors::RuntimeErrorKind;
use crate::errors::{EmptyChunkError, RuntimeError};
use crate::namespace::NameSpace;
use crate::object::string::{StringObject, STRING_TYPE};
use crate::{cast, isinstance, rc_refcell};
use crate::value::{Compare, Value};

type ValueStack = Rc<RefCell<Vec<StoredValue>>>;

pub struct VirtualMachine<'a> {
    chunk: StoredChunk,
    ip: usize, // instruction pointer
    debug_trace: bool,
    value_stack: ValueStack,
    globals: StoredNameSpace<'a>,
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

impl VirtualMachine<'_> {
    pub fn new(chunk: StoredChunk, debug_trace: bool) -> Result<Self, Error> {
        if chunk.borrow().is_empty() {
            return Err(EmptyChunkError {}.into());
        }
        Ok(Self {
            chunk,
            ip: 0,
            debug_trace,
            value_stack: rc_refcell!(vec![]),
            globals: rc_refcell!(NameSpace::new()),
        })
    }

    pub fn exec(&mut self) -> VoidResult {
        if self.debug_trace {
            println!("Executing this chunk:");
            println!("{}", self.chunk.borrow());
            println!()
        }
        loop {
            let borrowed_chunk = self.chunk.borrow();
            let Some(instruction) = borrowed_chunk.get(self.ip) else {
                return Ok(());
            };

            if self.debug_trace {
                println!("Current stack: {:?}", self.value_stack.borrow());
                println!("{instruction}");
            }

            match instruction.kind() {
                OpCodeKind::Const { const_idx } => {
                    self.op_const(*const_idx);
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
                OpCodeKind::Pop => { self.pop_or_err()?; },
                OpCodeKind::DefineGlobal { name_idx } => self.op_define_global(*name_idx)?
            }
            self.ip += 1;
        }
    }

    fn runtime_error(&self, kind: RuntimeErrorKind) -> Error {
        let borrowed_chunk = self.chunk.borrow();
        let Some(prev_instruction) = borrowed_chunk.get(self.ip - 1) else {
            panic!("Cannot get previous instruction");
        };

        RuntimeError {
            kind,
            line: prev_instruction.line(),
        }
        .into()
    }

    fn peek(&self) -> Result<StoredValue, Error> {
        let Some(value) = self.value_stack.borrow().last().cloned() else {
            return Err(self.runtime_error(RuntimeErrorKind::MissingValue));
        };
        Ok(value)
    }

    fn push_value(&self, value: Value) {
        self.value_stack.borrow_mut().push(rc_refcell!(value));
    }

    fn push_stored_value(&self, value: StoredValue) {
        self.value_stack.borrow_mut().push(value);
    }

    fn pop_or_err(&self) -> Result<StoredValue, Error> {
        let Some(value) = self.value_stack.borrow_mut().pop() else {
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

    fn bin_op(&self, kind: BinOpKind) -> VoidResult {
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
                    value: format!("between {} and {}", val1.type_name(), val2.type_name()),
                }));
            }
        }

        Ok(())
    }

    fn read_string_const(&self, idx: usize) -> Result<&StringObject, Error> {
        let borrowed_chunk = self.chunk.borrow();
        let const_value = borrowed_chunk.get_const(idx).unwrap();

        match &*const_value.borrow() {
            Value::Object(obj) => {
                if !isinstance!(obj, StringObject) {
                    Err(self.runtime_error(RuntimeErrorKind::TypeError { got: obj.type_name(), expected: STRING_TYPE.to_owned() }))
                } else {
                    Ok(cast!(obj => StringObject))
                }
            },
            _ => Err(self.runtime_error(RuntimeErrorKind::TypeError { got: "<not a string>".to_owned(), expected: STRING_TYPE.to_owned() }))
        }
    }

    fn op_const(&self, const_idx: usize) {
        let borrowed_chunk = self.chunk.borrow();
        let const_value = borrowed_chunk.get_const(const_idx).unwrap();
        if self.debug_trace {
            println!("Pushed const: {}", const_value.borrow());
        }
        self.push_stored_value(const_value);
    }

    fn op_negate(&self) -> VoidResult {
        let peek = self.peek()?;
        if !peek.borrow().support_negation() {
            return Err(self.runtime_error(RuntimeErrorKind::OperationNotSupported {
                op: "-".to_owned(),
                value: format!("for {}", peek.borrow()),
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

    fn op_cmp(&self, expected: Compare) -> VoidResult {
        let b = self.pop_or_err()?;
        let a = self.pop_or_err()?;

        let result = a.borrow().cmp(&b.borrow()) == expected;
        self.push_value(Value::Boolean(result));
        Ok(())
    }

    fn op_print(&self) -> VoidResult {
        let value = self.pop_or_err()?;
        println!("{}", value.borrow());
        Ok(())
    }

    fn op_define_global(&self, name_idx: usize) -> VoidResult {
        let name = self.read_string_const(name_idx)?;
        self.globals.borrow_mut().insert(name, self.peek()?);
        Ok(())
    }
}
