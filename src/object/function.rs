use std::fmt::Display;

use crate::{
    alias::{AnyObject, StoredChunk},
    chunk::Chunk,
    object::Object,
    rc_refcell,
    token::Literal,
};

pub const FUNCTION_TYPE: &str = "function";

#[derive(Debug)]
pub struct FunctionObject {
    pub arity: usize,
    pub chunk: StoredChunk,
    pub name: Literal,
}

impl FunctionObject {
    pub fn new(arity: usize, name: Literal) -> Self {
        Self {
            arity,
            chunk: rc_refcell!(Chunk::new()),
            name,
        }
    }
    pub fn incr_arity(&mut self) {
        self.arity += 1;
    }
}

impl Display for FunctionObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "'{}' {FUNCTION_TYPE} object", self.name)
    }
}

impl Object for FunctionObject {
    fn type_name(&self) -> String {
        String::from(FUNCTION_TYPE)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
