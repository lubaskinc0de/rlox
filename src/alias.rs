use std::{cell::RefCell, rc::Rc};

use crate::value::Value;

pub type StoredValue = Rc<RefCell<Value>>;
