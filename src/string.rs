use std::{any::Any, fmt::Display};

use crate::{
    alias::DynObject,
    object::Object,
    value::Compare,
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
}
