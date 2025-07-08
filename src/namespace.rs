use std::{collections::HashMap, rc::Rc};

use crate::alias::StoredValue;

type K = Rc<String>;
type V = StoredValue;

pub struct NameSpace {
    table: HashMap<K, V>,
}

impl NameSpace {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.table.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<StoredValue> {
        self.table.get(key).cloned()
    }
}
