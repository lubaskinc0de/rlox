use std::{cell::RefCell, rc::Rc};

use anyhow::Error;

use crate::{chunk::Chunk, object::Object, value::Value};

pub type StoredValue = Rc<RefCell<Value>>;
pub type StoredChunk = Rc<RefCell<Chunk>>;
pub type VoidResult = Result<(), Error>;
pub type StoredObject<T> = Rc<RefCell<T>>;
pub type AnyObject = Rc<RefCell<dyn Object>>;
