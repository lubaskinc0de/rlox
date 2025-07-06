use std::{borrow::Cow, fmt::Display};

use crate::{object::Object, value::Value};

#[derive(Debug, Clone)]
pub struct StringObject {
    value: String,
}

impl StringObject {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

impl Display for StringObject {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Object for StringObject {
    fn type_name(&self) -> String {
        String::from("string")
    }

    fn get_attribute(&self, attr_name: &str) -> Option<Cow<'_, Value>> {
        // for example only
        match attr_name {
            "length" => Some(Cow::Owned(Value::Float(self.value.len() as f64))),
            _ => None,
        }
    }

    fn copy(&self) -> Box<dyn Object> {
        Box::new(StringObject::new(self.value.clone()))
    }
}
