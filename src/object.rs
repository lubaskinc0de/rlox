use std::{
    any::Any,
    fmt::{Debug, Display},
};

use crate::{
    alias::{AnyObject, StoredValue},
    errors::RuntimeErrorKind,
    value::Compare,
};

pub mod function;
pub mod string;

pub type ResultRE<T> = Result<T, RuntimeErrorKind>; // result runtime error

pub trait Object: Debug + Display + Any {
    fn as_any(&self) -> &dyn Any;

    fn type_name(&self) -> String;

    #[allow(unused_variables, dead_code)]
    fn get_attribute(&self, attr_name: &str) -> Option<StoredValue> {
        None
    }

    fn copy(&self) -> AnyObject;

    #[allow(unused_variables)]
    fn cmp(&self, other: &AnyObject) -> ResultRE<Compare> {
        Ok(Compare::NotEqual)
    }

    fn operation_not_supported(&self, other: &AnyObject, op: String) -> RuntimeErrorKind {
        RuntimeErrorKind::OperationNotSupported {
            target: format!(
                "between {} and {}",
                self.type_name(),
                other.borrow().type_name()
            ),
            op,
        }
    }

    fn add(&self, other: &AnyObject) -> ResultRE<StoredValue> {
        Err(self.operation_not_supported(other, "+".to_owned()))
    }
    
}
