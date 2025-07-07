use std::any::Any;
use std::collections::HashMap;

use crate::{
    alias::StoredValue,
    cast,
    errors::RuntimeErrorKind,
    isinstance,
    object::{string::{StringObject, STRING_TYPE}, Object},
};

type K<'a> = &'a StringObject;
type V = StoredValue;

pub struct NameSpace<'key> {
    table: HashMap<K<'key>, V>,
}

impl<'key> NameSpace<'key> {
    pub fn new() -> Self {
        Self {
            table: HashMap::new(),
        }
    }

    pub fn insert(
        &mut self,
        key: K,
        value: V,
    ) -> Result<(), RuntimeErrorKind> {
        if !isinstance!(key, StringObject) {
            return Err(RuntimeErrorKind::TypeError {
                got: key.type_name(),
                expected: String::from(STRING_TYPE),
            });
        }
        let as_string = cast!(key => StringObject);
        self.table.insert(as_string, value);
        Ok(())
    }

    pub fn get(&mut self, key: K) -> Result<Option<StoredValue>, RuntimeErrorKind> {
        if !isinstance!(key, StringObject) {
            return Err(RuntimeErrorKind::TypeError {
                got: key.type_name(),
                expected: String::from(STRING_TYPE),
            });
        }

        Ok(self.table.get(cast!(key => StringObject)).cloned())
    }

    pub fn delete(&mut self, key: K) -> Result<(), RuntimeErrorKind> {
        if !isinstance!(key, StringObject) {
            return Err(RuntimeErrorKind::TypeError {
                got: key.type_name(),
                expected: String::from(STRING_TYPE),
            });
        }

        self.table.remove(cast!(key => StringObject));
        Ok(())
    }
}
