use std::{borrow::Cow, fmt::{Debug, Display}};

use crate::value::Value;

pub trait Object: Debug + Display {
    fn type_name(&self) -> String;
    fn get_attribute(&self, attr_name: &str) -> Option<Cow<'_, Value>>;
    fn copy(&self) -> Box<dyn Object>;
}
