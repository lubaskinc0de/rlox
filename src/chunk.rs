use std::{fmt::Display, rc::Rc, vec};

use crate::value::Value;

pub enum OpCode {
    OpReturn {
        line: usize,
    },
    OpConst {
        line: usize,
        const_idx: usize,
    },
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            OpCode::OpReturn { line } => format!("OP_RETURN     L{}", line),
            OpCode::OpConst { const_idx, line } => format!("OP_CONST {}     L{}", const_idx, line),
        };
        write!(f, "{}", repr)
    }
}

pub struct Chunk {
    code: Vec<OpCode>,
    constants: Vec<Rc<Value>>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            constants: vec![],
        }
    }

    pub fn push(&mut self, op_code: OpCode) -> () {
        self.code.push(op_code);
    }

    pub fn push_const(&mut self, value: Rc<Value>) -> usize {
        self.constants.push(value);
        self.constants.len() - 1  // index of const
    }

    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&OpCode> {
        return self.code.get(index)
    }

    pub fn get_const(&self, index: usize) -> Option<Rc<Value>> {
        self.constants.get(index).cloned()  // Rc clone
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.code.iter().peekable();
        let mut offset = 0;
        while let Some(op_code) = iter.next() {
            write!(f, "{}   {}", offset, op_code)?;
            if iter.peek().is_some() {
                write!(f, "\n")?;
                offset += 1;
            }
        }
        Ok(())
    }
}