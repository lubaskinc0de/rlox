use std::{any::Any, fmt::Display};

use crate::{
    alias::{DynObject, StoredValue},
    cast,
    errors::RuntimeErrorKind,
    isinstance,
    object::{Object, ResultRE},
    rc_refcell,
    value::{Compare, Value},
};

pub const STRING_TYPE: &str = "string";

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
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
        String::from(STRING_TYPE)
    }

    fn copy(&self) -> DynObject {
        Box::new(StringObject::new(self.value.clone()))
    }

    fn cmp(&self, other: &DynObject) -> ResultRE<Compare> {
        if !isinstance!(other, StringObject) {
            return Ok(Compare::NotEqual);
        }
        let as_string = cast!(other => StringObject)?;
        if as_string.value == self.value {
            Ok(Compare::Equal)
        } else {
            Ok(Compare::NotEqual)
        }
    }

    fn add(&self, other: &DynObject) -> ResultRE<StoredValue> {
        if !isinstance!(other, StringObject) {
            return Err(self.operation_not_supported(other, "+".to_owned()));
        }
        let as_string = cast!(other => StringObject)?;
        let mut concatenated_string = String::new();
        concatenated_string.push_str(&self.value);
        concatenated_string.push_str(&as_string.value);

        Ok(rc_refcell!(Value::Object(Box::new(StringObject::new(
            concatenated_string
        )))))
    }
}
