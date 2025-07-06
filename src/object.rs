use std::{
    any::Any,
    borrow::Cow,
    fmt::{Debug, Display},
};

use crate::value::{Compare, Value};

pub trait AnyObject: Object + Any {}

pub trait Object: Debug + Display {
    fn type_name(&self) -> String;
    fn get_attribute(&self, attr_name: &str) -> Option<Cow<'_, Value>>;
    fn copy(&self) -> Box<dyn AnyObject>;
    fn cmp(&self, other: &Box<dyn AnyObject>) -> Compare;
}
