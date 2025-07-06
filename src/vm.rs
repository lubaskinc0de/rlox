use std::cell::RefCell;
use std::rc::Rc;

use anyhow::Error;

use crate::alias::{StoredChunk, StoredValue};
use crate::bin_op::BinOpKind;
use crate::chunk::OpCodeKind;
use crate::errors::RuntimeErrorKind;
use crate::errors::{EmptyChunkError, RuntimeError};
use crate::value::Value;
use crate::rc_refcell;

type ValueStack = Rc<RefCell<Vec<StoredValue>>>;

pub struct VirtualMachine {
    chunk: StoredChunk,
    ip: usize,
    debug_trace: bool,
    value_stack: ValueStack,
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
    pub fn new(chunk: StoredChunk, debug_trace: bool) -> Result<Self, Error> {
        if chunk.borrow().is_empty() {
            return Err(EmptyChunkError {}.into());
        }
        Ok(Self {
            chunk,
            ip: 0,
            debug_trace,
            value_stack: rc_refcell!(vec![]),
        })
    }

    pub fn exec(&mut self) -> Result<(), Error> {
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
                println!("{instruction}");
            }

            match instruction.kind() {
                OpCodeKind::Const { const_idx } => {
                    let const_value = borrowed_chunk.get_const(*const_idx).unwrap();
                    if self.debug_trace {
                        println!("Pushed const: {}", const_value.borrow());
                    }
                    self.value_stack.borrow_mut().push(const_value);
                }
                OpCodeKind::Negate => {
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
                            self.value_stack
                                .borrow_mut()
                                .push(rc_refcell!(Value::Float(-float_value)));
                        }
                        _ => unreachable!(),
                    }
                }
                OpCodeKind::Add => self.bin_op(BinOpKind::Add)?,
                OpCodeKind::Sub => self.bin_op(BinOpKind::Sub)?,
                OpCodeKind::Mul => self.bin_op(BinOpKind::Mul)?,
                OpCodeKind::Div => self.bin_op(BinOpKind::Div)?,
                OpCodeKind::Null => {
                    self.value_stack.borrow_mut().push(rc_refcell!(Value::Null));
                },
                OpCodeKind::True => {
                    self.value_stack.borrow_mut().push(rc_refcell!(Value::Boolean(true)));
                },
                OpCodeKind::False => {
                    self.value_stack.borrow_mut().push(rc_refcell!(Value::Boolean(false)));
                },
            }
            self.ip += 1;
        }
    }

    pub fn stack_top(&self) -> Option<StoredValue> {
        return self.value_stack.borrow().last().cloned();
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

    fn pop_or_err(&self) -> Result<StoredValue, Error> {
        let Some(value) = self.value_stack.borrow_mut().pop() else {
            return Err(self.runtime_error(RuntimeErrorKind::MissingValue));
        };
        Ok(value)
    }

    fn bin_op(&self, kind: BinOpKind) -> Result<(), Error> {
        let b = self.pop_or_err()?;
        let a = self.pop_or_err()?;

        match (&*a.borrow(), &*b.borrow()) {
            (Value::Float(a_val), Value::Float(b_val)) => {
                let calculated = calc!(a_val, b_val, kind.to_string().as_str());
                self.value_stack
                    .borrow_mut()
                    .push(rc_refcell!(Value::Float(calculated)));
            }
            (val1, val2) => {
                return Err(self.runtime_error(RuntimeErrorKind::OperationNotSupported {
                    op: "-".to_owned(),
                    value: format!("between {} and {}", val1.type_name(), val2.type_name()),
                }));
            }
        }

        Ok(())
    }
}
