use std::{cell::RefCell, rc::Rc};

use anyhow::Error;

use crate::{chunk::Chunk, value::Value};

pub type StoredValue = Rc<RefCell<Value>>;
pub type StoredChunk = Rc<RefCell<Chunk>>;
pub type VoidResult = Result<(), Error>;
