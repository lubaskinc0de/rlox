use std::{
    any::Any,
    fmt::{Debug, Display},
};

use crate::{
    alias::{DynObject, StoredValue},
    errors::RuntimeErrorKind,
    value::Compare,
};

pub mod string;

pub trait Object: Debug + Display + Any {
    fn type_name(&self) -> String;

    #[allow(unused_variables, dead_code)]
    fn get_attribute(&self, attr_name: &str) -> Option<StoredValue> {
        None
    }

    fn copy(&self) -> DynObject;

    #[allow(unused_variables)]
    fn cmp(&self, other: &DynObject) -> Compare {
        Compare::NotEqual
    }

    fn operation_not_supported(&self, other: &DynObject, op: String) -> RuntimeErrorKind {
        RuntimeErrorKind::OperationNotSupported {
            value: format!("between {} and {}", self.type_name(), other.type_name()),
            op,
        }
    }

    fn add(&self, other: &DynObject) -> Result<StoredValue, RuntimeErrorKind> {
        Err(self.operation_not_supported(other, "+".to_owned()))
    }
}
