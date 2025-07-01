use std::fmt::Display;

use crate::bin_op::BinOpKind;

#[derive(Debug)]
pub enum Value {
    Float(f64),
}

impl Value {
    #[allow(unused_variables)]
    pub fn is_supported_binop(&self, kind: &BinOpKind) -> bool {
        match self {
            Value::Float(_) => true,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            Value::Float(value) => format!("<value {value} of type float>"),
        };
        write!(f, "{repr}")
    }
}
