use std::rc::Rc;

use anyhow::Error;

use crate::{chunk::Chunk, errors::EmptyChunkError};
use crate::OpCode;

pub struct VirtualMachine {
    chunk: Rc<Chunk>,
    ip: usize,
    debug_trace: bool,
}

impl VirtualMachine {
    pub fn new(chunk: Rc<Chunk>, debug_trace: bool) -> Result<Self, Error> {
        if chunk.is_empty() {
            return Err(EmptyChunkError {}.into());
        }
        Ok(Self {
            chunk,
            ip: 0,
            debug_trace,
        })
    }

    pub fn exec_chunk(&mut self) -> Result<(), Error> {
        loop {
            let Some(instruction) = self.chunk.get(self.ip) else {
                return Ok(())
            };
            println!("{}", instruction);
            match instruction {
                OpCode::OpReturn { line: _ } => (),
                OpCode::OpConst { line: _, const_idx } => {
                    let const_value = self.chunk.get_const(*const_idx);
                    println!("{:?}", const_value)
                },
            }
            self.ip += 1;
        }
    }
}
