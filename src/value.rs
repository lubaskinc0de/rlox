use std::fmt::Display;


#[derive(Debug)]
pub enum Value {
    Float(f64),
    Boolean(bool),
    Null,
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
        match self {
            Value::Float(_) => true,
            _ => false,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(false) | Value::Null => false,
            _ => true,
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
