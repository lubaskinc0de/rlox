use std::{fmt::Display, vec};

use crate::alias::StoredValue;

const STACK_CAPACITY: usize = 256;

#[derive(Debug)]
pub enum OpCodeKind {
    Const { const_idx: usize },
    Negate,
    Add,
    Sub,
    Mul,
    Div,
    Null,
    True,
    False,
    Not,
    Eq,
    Gt,
    Lt,
    Print,
    Pop,
    DefineGlobal { name_idx: usize },
    ReadGlobal { name_idx: usize },
    SetGlobal { name_idx: usize },
}

impl Display for OpCodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (name, args) = match self {
            OpCodeKind::Const { const_idx } => ("OP_CONST", format!("{const_idx}")),
            OpCodeKind::Negate => ("OP_NEGATE", "".to_string()),
            OpCodeKind::Add => ("OP_ADD", "".to_string()),
            OpCodeKind::Sub => ("OP_SUB", "".to_string()),
            OpCodeKind::Mul => ("OP_MUL", "".to_string()),
            OpCodeKind::Div => ("OP_DIV", "".to_string()),
            OpCodeKind::Null => ("OP_NULL", "".to_string()),
            OpCodeKind::False => ("OP_FALSE", "".to_string()),
            OpCodeKind::True => ("OP_TRUE", "".to_string()),
            OpCodeKind::Not => ("OP_NOT", "".to_string()),
            OpCodeKind::Eq => ("OP_EQ", "".to_string()),
            OpCodeKind::Gt => ("OP_GT", "".to_string()),
            OpCodeKind::Lt => ("OP_LT", "".to_string()),
            OpCodeKind::Print => ("OP_PRINT", "".to_string()),
            OpCodeKind::Pop => ("OP_POP", "".to_string()),
            OpCodeKind::DefineGlobal { name_idx } => ("OP_DEFINE_GLOBAL", format!("{name_idx}")),
            OpCodeKind::ReadGlobal { name_idx } => ("OP_READ_GLOBAL", format!("{name_idx}")),
            OpCodeKind::SetGlobal { name_idx } => ("OP_SET_GLOBAL", format!("{name_idx}")),
        };

        write!(f, "{name:<12} {args:<6}")
    }
}

#[derive(Debug)]
pub struct OpCode {
    kind: OpCodeKind,
    line: usize,
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} L{}", self.kind, self.line())
    }
}

impl OpCode {
    pub fn new(kind: OpCodeKind, line: usize) -> Self {
        Self { kind, line }
    }
    pub fn line(&self) -> usize {
        self.line
    }
    pub fn kind(&self) -> &OpCodeKind {
        &self.kind
    }
}

#[derive(Debug)]
pub struct Chunk {
    code: Vec<OpCode>,
    pub constants: Vec<StoredValue>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: vec![],
            constants: Vec::with_capacity(STACK_CAPACITY),
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

    pub fn get_const(&self, index: usize) -> Option<&StoredValue> {
        self.constants.get(index)
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
