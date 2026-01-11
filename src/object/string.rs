use std::{fmt::Display, rc::Rc};

use crate::{
    alias::{AnyObject, StoredValue},
    cast,
    errors::RuntimeErrorKind,
    isinstance,
    object::{Object, ResultRE},
    rc_refcell,
    token::Literal,
    value::{Compare, Value},
};

pub const STRING_TYPE: &str = "string";

#[derive(Debug)]
pub struct StringObject {
    pub value: Literal,
}

impl StringObject {
    pub fn new(value: Literal) -> Self {
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

    fn copy(&self) -> AnyObject {
        rc_refcell!(StringObject::new(self.value.clone()))
    }

    fn cmp(&self, other: &AnyObject) -> ResultRE<Compare> {
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

    fn add(&self, other: &AnyObject) -> ResultRE<StoredValue> {
        if !isinstance!(other, StringObject) {
            return Err(self.operation_not_supported(other, "+".to_owned()));
        }
        let as_string = cast!(other => StringObject)?;
        let mut concatenated_string = String::new();
        concatenated_string.push_str(&self.value);
        concatenated_string.push_str(&as_string.value);

        Ok(rc_refcell!(Value::Object(rc_refcell!(StringObject::new(
            Rc::new(concatenated_string)
        )))))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
