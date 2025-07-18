use std::fmt::Display;

use crate::{alias::DynObject, object::ResultRE, token::Literal};

#[derive(Debug)]
pub enum Value {
    Float(f64),
    Boolean(bool),
    Null,
    Identifier(Literal),
    Object(DynObject),
}

#[derive(PartialEq, Debug)]
pub enum Compare {
    Equal,
    NotEqual,
    Greater,
    Lower,
}

impl Value {
    pub fn type_name(&self) -> String {
        match self {
            Value::Float(_) => "float".to_owned(),
            Value::Boolean(_) => "boolean".to_owned(),
            Value::Null => "null".to_owned(),
            Value::Object(obj) => obj.type_name(),
            Value::Identifier(_) => "identifier".to_owned(),
        }
    }

    pub fn support_negation(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    pub fn as_bool(&self) -> bool {
        !matches!(self, Value::Boolean(false) | Value::Null)
    }

    pub fn cmp(&self, other: &Value) -> ResultRE<Compare> {
        match (&self, other) {
            (Value::Float(a), Value::Float(b)) => {
                if a > b {
                    Ok(Compare::Greater)
                } else if a < b {
                    Ok(Compare::Lower)
                } else {
                    Ok(Compare::Equal)
                }
            }
            (Value::Boolean(a), Value::Boolean(b)) => {
                if a == b {
                    Ok(Compare::Equal)
                } else {
                    Ok(Compare::NotEqual)
                }
            }
            (Value::Null, Value::Null) => Ok(Compare::Equal),
            (Value::Object(a), Value::Object(b)) => a.cmp(b),
            _ => Ok(Compare::NotEqual),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            Value::Float(value) => format!("{value}"),
            Value::Boolean(value) => format!("{value}"),
            Value::Null => "null".to_owned(),
            Value::Object(obj) => format!("{obj}"),
            Value::Identifier(val) => format!("<value '{val}' of type identifier>"),
        };
        write!(f, "{repr}")
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Float(val) => Value::Float(*val),
            Value::Boolean(val) => Value::Boolean(*val),
            Value::Null => Value::Null,
            Value::Object(object) => Value::Object(object.copy()),
            Value::Identifier(val) => Value::Identifier(val.clone()),
        }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (&self, other) {
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Null, Value::Null) => true,
            (Value::Object(a), Value::Object(b)) => {
                a.cmp(b).unwrap_or(Compare::NotEqual) == Compare::Equal
            }
            _ => false,
        }
    }
}

impl Eq for Value {}
