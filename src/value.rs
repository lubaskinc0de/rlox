use std::fmt::Display;

#[derive(Debug)]
pub enum Value {
    Float(f64),
    Boolean(bool),
    Null,
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
            _ => Compare::NotEqual,
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let repr = match self {
            Value::Float(value) => format!("<value {value} of type {}>", self.type_name()),
            Value::Boolean(value) => format!("<value {value} of type {}>", self.type_name()),
            Value::Null => "null".to_owned(),
        };
        write!(f, "{repr}")
    }
}
