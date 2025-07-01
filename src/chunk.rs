use std::{fmt::Display, vec};

use crate::alias::StoredValue;

pub enum OpCode {
    OpConst { line: usize, const_idx: usize },
    OpNegate { line: usize },
    OpAdd { line: usize },
    OpSub { line: usize },
    OpMul { line: usize },
    OpDiv { line: usize },
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, args, line) = match self {
            OpCode::OpConst { const_idx, line } => ("OP_CONST", format!("{const_idx}"), line),
            OpCode::OpNegate { line } => ("OP_NEGATE", "".to_string(), line),
            OpCode::OpAdd { line } => ("OP_ADD", "".to_string(), line),
            OpCode::OpSub { line } => ("OP_SUB", "".to_string(), line),
            OpCode::OpMul { line } => ("OP_MUL", "".to_string(), line),
            OpCode::OpDiv { line } => ("OP_DIV", "".to_string(), line),
        };

        write!(f, "{name:<12} {args:<6} L{line}")
    }
}

pub struct Chunk {
    code: Vec<OpCode>,
    constants: Vec<StoredValue>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            constants: vec![],
        }
    }

    pub fn push(&mut self, op_code: OpCode) {
        self.code.push(op_code);
    }

    pub fn push_const(&mut self, value: StoredValue) -> usize {
        self.constants.push(value);
        self.constants.len() - 1 // index of const
    }

    pub fn is_empty(&self) -> bool {
        self.code.is_empty()
    }

    pub fn get(&self, index: usize) -> Option<&OpCode> {
        self.code.get(index)
    }

    pub fn get_const(&self, index: usize) -> Option<StoredValue> {
        self.constants.get(index).cloned() // Rc clone
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.code.iter().peekable();
        let mut offset = 0;
        while let Some(op_code) = iter.next() {
            write!(f, "{offset}   {op_code}")?;
            if iter.peek().is_some() {
                writeln!(f)?;
                offset += 1;
            }
        }
        Ok(())
    }
}
