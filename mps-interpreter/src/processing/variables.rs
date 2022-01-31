use std::fmt::{Debug, Display, Error, Formatter};

use std::collections::HashMap;

use crate::lang::MpsOp;
use crate::lang::MpsTypePrimitive;
use crate::lang::RuntimeMsg;

#[derive(Debug)]
pub enum MpsType {
    Op(Box<dyn MpsOp>),
    Primitive(MpsTypePrimitive),
}

impl Display for MpsType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Op(op) => write!(f, "Op({})", op),
            Self::Primitive(p) => write!(f, "{}", p),
        }
    }
}

pub trait MpsVariableStorer: Debug {
    fn get(&self, name: &str) -> Result<&'_ MpsType, RuntimeMsg> {
        match self.get_opt(name) {
            Some(item) => Ok(item),
            None => Err(RuntimeMsg(format!("Variable '{}' not found", name))),
        }
    }

    fn get_opt(&self, name: &str) -> Option<&'_ MpsType>;

    fn get_mut(&mut self, name: &str) -> Result<&'_ mut MpsType, RuntimeMsg> {
        match self.get_mut_opt(name) {
            Some(item) => Ok(item),
            None => Err(RuntimeMsg(format!("Variable '{}' not found", name))),
        }
    }

    fn get_mut_opt(&mut self, name: &str) -> Option<&'_ mut MpsType>;

    fn assign(&mut self, name: &str, value: MpsType) -> Result<(), RuntimeMsg>;

    fn declare(&mut self, name: &str, value: MpsType) -> Result<(), RuntimeMsg>;

    fn remove(&mut self, name: &str) -> Result<MpsType, RuntimeMsg>;

    fn exists(&self, name: &str) -> bool {
        self.get_opt(name).is_some()
    }
}

#[derive(Default, Debug)]
pub struct MpsOpStorage {
    storage: HashMap<String, MpsType>,
}

impl MpsVariableStorer for MpsOpStorage {
    fn get_opt(&self, key: &str) -> Option<&MpsType> {
        self.storage.get(key)
    }

    fn get_mut_opt(&mut self, key: &str) -> Option<&mut MpsType> {
        self.storage.get_mut(key)
    }

    fn assign(&mut self, key: &str, item: MpsType) -> Result<(), RuntimeMsg> {
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

    fn declare(&mut self, key: &str, item: MpsType) -> Result<(), RuntimeMsg> {
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

    fn remove(&mut self, key: &str) -> Result<MpsType, RuntimeMsg> {
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
