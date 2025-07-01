use std::{cell::RefCell, rc::Rc};

use crate::{chunk::Chunk, value::Value};

pub type StoredValue = Rc<RefCell<Value>>;
pub type StoredChunk = Rc<RefCell<Chunk>>;
