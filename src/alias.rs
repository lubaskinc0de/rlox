use std::{cell::RefCell, rc::Rc};

use anyhow::Error;

use crate::{chunk::Chunk, namespace::NameSpace, object::Object, value::Value};

pub type StoredValue = Rc<RefCell<Value>>;
pub type StoredChunk = Rc<RefCell<Chunk>>;
pub type VoidResult = Result<(), Error>;
pub type DynObject = Box<dyn Object>;
pub type StoredNameSpace = Rc<RefCell<NameSpace>>;
