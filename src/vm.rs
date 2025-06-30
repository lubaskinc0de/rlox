use anyhow::Error;

use crate::alias::StoredValue;
use crate::{rc_refcell, OpCode};
use crate::errors::RuntimeError;
use crate::value::Value;
use crate::{chunk::Chunk, errors::EmptyChunkError};

pub struct VirtualMachine {
    chunk: Chunk,
    ip: usize,
    debug_trace: bool,
    value_stack: Vec<StoredValue>,
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
                            self.value_stack.push(rc_refcell!(Value::Float(-float_value)));
                        }
                    }
                }
            }
            self.ip += 1;
        }
    }

    pub fn stack_top(&self) -> Option<StoredValue> {
        return self.value_stack.last().cloned()
    }

    fn pop_or_err(&mut self) -> Result<StoredValue, Error> {
        let Some(value) = self.value_stack.pop() else {
            return Err(RuntimeError::MissingValue.into());
        };
        Ok(value)
    }
}
