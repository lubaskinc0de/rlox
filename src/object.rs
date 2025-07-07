use std::{
    any::Any,
    fmt::{Debug, Display},
};

use crate::{
    alias::{DynObject, StoredValue},
    value::Compare,
};

pub trait Object: Debug + Display + Any {
    fn type_name(&self) -> String;

    #[allow(unused_variables, dead_code)]
    fn get_attribute(&self, attr_name: &str) -> Option<StoredValue> {
        None
    }

    fn copy(&self) -> DynObject;

    fn cmp(&self, other: &DynObject) -> Compare;
}
