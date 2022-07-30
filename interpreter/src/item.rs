use std::collections::HashMap;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::TypePrimitive;
//use crate::lang::RuntimeError;
//use crate::processing::OpGetter;

/// general type object for MPS
#[derive(Clone, Debug, Default)]
pub struct Item {
    fields: HashMap<String, TypePrimitive>,
}

impl Item {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn field(&self, name: &str) -> Option<&'_ TypePrimitive> {
        self.fields.get(name)
    }

    pub fn set_field(&mut self, name: &str, value: TypePrimitive) -> Option<TypePrimitive> {
        self.fields.insert(name.to_owned(), value)
    }

    pub fn set_field_chain(&mut self, name: &str, value: TypePrimitive) -> &mut Self {
        self.set_field(name, value);
        self
    }

    pub fn set_field_chain2(mut self, name: &str, value: TypePrimitive) -> Self {
        self.set_field(name, value);
        self
    }

    pub fn remove_field(&mut self, name: &str) -> Option<TypePrimitive> {
        self.fields.remove(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = &String> {
        self.fields.keys()
    }

    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }
}

impl Display for Item {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "Item[({} fields)]", self.fields.len())
    }
}

impl std::hash::Hash for Item {
    fn hash<H>(&self, state: &mut H)
    where
        H: std::hash::Hasher,
    {
        // hashing is order-dependent, so the pseudo-random sorting of HashMap keys
        // prevents it from working correctly without sorting
        let mut keys: Vec<_> = self.fields.keys().collect();
        keys.as_mut_slice().sort();
        for key in keys {
            let val = self.fields.get(key).unwrap();
            key.hash(state);
            val.hash(state);
        }
    }
}

impl std::cmp::PartialEq for Item {
    /*fn eq(&self, other: &Self) -> bool {
        for (key, val) in self.fields.iter() {
            if let Some(other_val) = other.fields.get(key) {
                if other_val != val {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }*/

    fn eq(&self, other: &Self) -> bool {
        self.fields == other.fields
    }
}

impl std::cmp::Eq for Item {}

/*pub(crate) trait ItemRuntimeUtil {
    fn get_field_runtime(&self, name: &str, op: &mut OpGetter) -> Result<&TypePrimitive, RuntimeError>;
}

impl ItemRuntimeUtil for Item {
    fn get_field_runtime(&self, name: &str, op: &mut OpGetter) -> Result<&TypePrimitive, RuntimeError> {
        match self.field(name) {
            Some(item) => Ok(item),
            None => Err(RuntimeError{
                line: 0,
                op: op(),
                msg: format!("Field {} not found on item", name),
            })
        }
    }
}*/
