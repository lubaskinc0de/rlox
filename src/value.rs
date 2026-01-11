use std::fmt::Display;

use crate::{alias::AnyObject, object::ResultRE, token::Literal};

#[derive(Debug)]
pub enum Value {
    Number(f64),
    Bool(bool),
    Null,
    Identifier(Literal),
    Object(AnyObject),
}

impl Value {
    pub fn type_name(&self) -> String {
        match self {
            Value::Number(_) => "number".to_owned(),
            Value::Bool(_) => "boolean".to_owned(),
            Value::Null => "null".to_owned(),
            Value::Object(obj) => obj.borrow().type_name(),
            Value::Identifier(_) => "identifier".to_owned(),
        }
    }

    pub fn is_negation_supported(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn as_bool(&self) -> bool {
        !matches!(self, Value::Bool(false) | Value::Null)
    }

    pub fn as_f64(&self) -> f64 {
        match &self {
            Value::Number(num) => *num,
            _ => panic!("as_f64() called on a non-number value"),
        }
    }

    pub fn cmp(&self, other: &Value) -> ResultRE<Compare> {
        match (&self, other) {
            (Value::Number(a), Value::Number(b)) => {
                if a > b {
                    Ok(Compare::Greater)
                } else if a < b {
                    Ok(Compare::Lower)
                } else {
                    Ok(Compare::Equal)
                }
            }
            (Value::Bool(a), Value::Bool(b)) => {
                if a == b {
                    Ok(Compare::Equal)
                } else {
                    Ok(Compare::NotEqual)
                }
            }
            (Value::Null, Value::Null) => Ok(Compare::Equal),
            (Value::Object(a), Value::Object(b)) => a.borrow().cmp(b),
            _ => Ok(Compare::NotEqual),
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Number(f)
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            Value::Number(value) => format!("{value}"),
            Value::Bool(value) => format!("{value}"),
            Value::Null => "null".to_owned(),
            Value::Object(obj) => format!("{}", obj.borrow()),
            Value::Identifier(val) => format!("'{val}'"),
        };
        write!(f, "{repr}")
    }
}

#[derive(PartialEq, Debug)]
pub enum Compare {
    Equal,
    NotEqual,
    Greater,
    Lower,
}
