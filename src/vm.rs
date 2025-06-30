use anyhow::Error;

use crate::alias::StoredValue;
use crate::bin_op::BinOpKind;
use crate::errors::RuntimeError;
use crate::value::Value;
use crate::{OpCode, rc_refcell};
use crate::{chunk::Chunk, errors::EmptyChunkError};

pub struct VirtualMachine {
    chunk: Chunk,
    ip: usize,
    debug_trace: bool,
    value_stack: Vec<StoredValue>,
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
    pub fn new(chunk: Chunk, debug_trace: bool) -> Result<Self, Error> {
        if chunk.is_empty() {
            return Err(EmptyChunkError {}.into());
        }
        Ok(Self {
            chunk,
            ip: 0,
            debug_trace,
            value_stack: vec![],
        })
    }

    pub fn exec(&mut self) -> Result<(), Error> {
        if self.debug_trace {
            println!("Executing this chunk:");
            println!("{}", self.chunk);
            println!()
        }
        loop {
            let Some(instruction) = self.chunk.get(self.ip) else {
                return Ok(());
            };

            if self.debug_trace {
                println!("{}", instruction);
                println!("Current stack: {:?}", self.value_stack)
            }

            match instruction {
                OpCode::OpReturn { line: _ } => (),
                OpCode::OpConst { line: _, const_idx } => {
                    let const_value = self.chunk.get_const(*const_idx).unwrap();
                    self.value_stack.push(const_value);
                }
                OpCode::OpNegate { line: _ } => {
                    let value = self.pop_or_err()?;
                    match &*value.borrow() {
                        Value::Float(float_value) => {
                            self.value_stack
                                .push(rc_refcell!(Value::Float(-float_value)));
                        }
                    }
                }
                OpCode::OpAdd { line: _ } => self.bin_op(BinOpKind::ADD)?,
                OpCode::OpSub { line: _ } => self.bin_op(BinOpKind::SUB)?,
                OpCode::OpMul { line: _ } => self.bin_op(BinOpKind::MUL)?,
                OpCode::OpDiv { line: _ } => self.bin_op(BinOpKind::DIV)?,
            }
            self.ip += 1;
        }
    }

    pub fn stack_top(&self) -> Option<StoredValue> {
        return self.value_stack.last().cloned();
    }

    fn pop_or_err(&mut self) -> Result<StoredValue, Error> {
        let Some(value) = self.value_stack.pop() else {
            return Err(RuntimeError::MissingValue.into());
        };
        Ok(value)
    }

    fn bin_op(&mut self, kind: BinOpKind) -> Result<(), Error> {
        let b = self.pop_or_err()?;
        let a = self.pop_or_err()?;

        if !a.borrow().is_supported_binop(&kind) {
            return Err(RuntimeError::OperationNotSupported {
                value_type: a.borrow().to_string(),
                operation_type: kind.to_string(),
            }
            .into());
        }

        if !b.borrow().is_supported_binop(&kind) {
            return Err(RuntimeError::OperationNotSupported {
                value_type: b.borrow().to_string(),
                operation_type: kind.to_string(),
            }
            .into());
        }

        match (&*a.borrow(), &*b.borrow()) {
            (Value::Float(a_val), Value::Float(b_val)) => {
                let calculated = calc!(a_val, b_val, kind.to_string().as_str());
                self.value_stack.push(rc_refcell!(Value::Float(calculated)));
            }
        }

        Ok(())
    }
}
