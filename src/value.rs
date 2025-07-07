use std::fmt::Display;

use crate::alias::DynObject;

#[derive(Debug)]
pub enum Value {
    Float(f64),
    Boolean(bool),
    Null,
    Object(DynObject),
}

#[derive(PartialEq)]
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
        }
    }

    pub fn support_negation(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    pub fn as_bool(&self) -> bool {
        !matches!(self, Value::Boolean(false) | Value::Null)
    }

    pub fn cmp(&self, other: &Value) -> Compare {
        match (&self, other) {
            (Value::Float(a), Value::Float(b)) => {
                if a > b {
                    Compare::Greater
                } else if a < b {
                    Compare::Lower
                } else {
                    Compare::Equal
                }
            }
            (Value::Boolean(a), Value::Boolean(b)) => {
                if a == b {
                    Compare::Equal
                } else {
                    Compare::NotEqual
                }
            }
            (Value::Null, Value::Null) => Compare::Equal,
            (Value::Object(a), Value::Object(b)) => a.cmp(b),
            _ => Compare::NotEqual,
        }
    }

    pub fn object(&self) -> Option<&DynObject> {
        match self {
            Value::Object(obj) => Some(obj),
            _ => None,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            Value::Float(value) => format!("<value {value} of type {}>", self.type_name()),
            Value::Boolean(value) => format!("<value {value} of type {}>", self.type_name()),
            Value::Null => "null".to_owned(),
            Value::Object(obj) => format!("<object {obj} of type {}>", self.type_name()),
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
        }
    }
}

impl Drop for Value {
    fn drop(&mut self) {
        println!("Value is dropped {self}")
    }
}
