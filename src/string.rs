use std::{any::Any, fmt::Display};

use crate::{
    alias::{DynObject, StoredValue},
    errors::RuntimeErrorKind,
    object::Object,
    rc_refcell,
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

    fn copy(&self) -> DynObject {
        Box::new(StringObject::new(self.value.clone()))
    }

    fn cmp(&self, other: &DynObject) -> Compare {
        if other.type_name() != self.type_name() {
            return Compare::NotEqual;
        }
        let obj = other.as_ref() as &dyn Any;
        match obj.downcast_ref::<StringObject>() {
            Some(as_string) => {
                if as_string.value == self.value {
                    Compare::Equal
                } else {
                    Compare::NotEqual
                }
            }
            None => unreachable!(),
        }
    }

    fn add(&self, other: &DynObject) -> Result<StoredValue, RuntimeErrorKind> {
        if other.type_name() != self.type_name() {
            return Err(self.operation_not_supported(other, "+".to_owned()));
        }
        let obj = other.as_ref() as &dyn Any;
        match obj.downcast_ref::<StringObject>() {
            Some(as_string) => {
                let mut concatenated_string = String::new();
                concatenated_string.push_str(&self.value);
                concatenated_string.push_str(&as_string.value);

                Ok(rc_refcell!(Value::Object(Box::new(StringObject::new(
                    concatenated_string
                )))))
            }
            None => unreachable!(),
        }
    }
}
