use std::{any::Any, borrow::Cow, fmt::Display};

use crate::{
    object::{AnyObject, Object},
    value::{Compare, Value},
};

#[derive(Debug, Clone)]
pub struct StringObject {
    pub value: String,
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

    fn copy(&self) -> Box<dyn AnyObject> {
        Box::new(StringObject::new(self.value.clone()))
    }

    fn cmp(&self, other: &Box<dyn AnyObject>) -> Compare {
        if other.type_name() != self.type_name() {
            return Compare::NotEqual;
        }
        let t = other as &dyn Any;
        match t.downcast_ref::<StringObject>() {
            Some(as_string) => {
                if as_string.value == self.value {
                    return Compare::Equal;
                } else {
                    return Compare::NotEqual;
                }
            }
            None => unreachable!(),
        }
    }
}

impl AnyObject for StringObject {}
