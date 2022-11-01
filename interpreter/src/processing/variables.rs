use std::fmt::{Debug, Display, Error, Formatter};

use std::collections::HashMap;

use crate::lang::Op;
use crate::lang::RuntimeMsg;
use crate::lang::TypePrimitive;
use crate::Item;

#[derive(Debug)]
pub enum Type {
    Op(Box<dyn Op>),
    Primitive(TypePrimitive),
    Item(Item),
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Op(op) => write!(f, "Op({})", op),
            Self::Primitive(p) => write!(f, "{}", p),
            Self::Item(item) => write!(f, "{}", item),
        }
    }
}

impl Type {
    #[inline(always)]
    pub const fn empty() -> Self {
        Type::Primitive(TypePrimitive::Empty)
    }
}

pub trait VariableStorer: Debug {
    fn get(&self, name: &str) -> Result<&'_ Type, RuntimeMsg> {
        match self.get_opt(name) {
            Some(item) => Ok(item),
            None => Err(RuntimeMsg(format!("Variable '{}' not found", name))),
        }
    }

    fn get_opt(&self, name: &str) -> Option<&'_ Type>;

    fn get_mut(&mut self, name: &str) -> Result<&'_ mut Type, RuntimeMsg> {
        match self.get_mut_opt(name) {
            Some(item) => Ok(item),
            None => Err(RuntimeMsg(format!("Variable '{}' not found", name))),
        }
    }

    fn get_mut_opt(&mut self, name: &str) -> Option<&'_ mut Type>;

    fn assign(&mut self, name: &str, value: Type) -> Result<(), RuntimeMsg>;

    fn declare(&mut self, name: &str, value: Type) -> Result<(), RuntimeMsg>;

    fn remove(&mut self, name: &str) -> Result<Type, RuntimeMsg>;

    fn swap(&mut self, name: &str, value: Option<Type>) -> Option<Type>;

    fn exists(&self, name: &str) -> bool {
        self.get_opt(name).is_some()
    }
}

#[derive(Default, Debug)]
pub struct OpStorage {
    storage: HashMap<String, Type>,
}

impl VariableStorer for OpStorage {
    fn get_opt(&self, key: &str) -> Option<&Type> {
        self.storage.get(key)
    }

    fn get_mut_opt(&mut self, key: &str) -> Option<&mut Type> {
        self.storage.get_mut(key)
    }

    fn assign(&mut self, key: &str, item: Type) -> Result<(), RuntimeMsg> {
        if !self.storage.contains_key(key) {
            Err(RuntimeMsg(format!(
                "Cannot assign to non-existent variable '{}'",
                key
            )))
        } else {
            self.storage.insert(key.to_string(), item);
            Ok(())
        }
    }

    fn declare(&mut self, key: &str, item: Type) -> Result<(), RuntimeMsg> {
        if self.storage.contains_key(key) {
            Err(RuntimeMsg(format!(
                "Cannot overwrite existing variable '{}'",
                key
            )))
        } else {
            self.storage.insert(key.to_string(), item);
            Ok(())
        }
    }

    fn swap(&mut self, key: &str, item: Option<Type>) -> Option<Type> {
        if let Some(item) = item {
            self.storage.insert(key.to_string(), item)
        } else {
            None
        }
    }

    fn remove(&mut self, key: &str) -> Result<Type, RuntimeMsg> {
        if self.storage.contains_key(key) {
            Ok(self.storage.remove(key).unwrap())
        } else {
            Err(RuntimeMsg(format!(
                "Cannot remove non-existing variable '{}'",
                key
            )))
        }
    }
}
