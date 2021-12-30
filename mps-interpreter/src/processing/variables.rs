use std::fmt::{Debug, Display, Error, Formatter};

use std::collections::HashMap;

use crate::lang::RuntimeError;
use crate::lang::MpsOp;
use crate::lang::MpsTypePrimitive;

use super::OpGetter;

#[derive(Debug)]
pub enum MpsType {
    Op(Box<dyn MpsOp>),
    Primitive(MpsTypePrimitive)
}

impl Display for MpsType {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        match self {
            Self::Op(op) => write!(f, "Op({})", op),
            Self::Primitive(p) => write!(f, "{}", p)
        }
    }
}

pub trait MpsVariableStorer: Debug {
    fn get(&self, name: &str, op: &mut OpGetter) -> Result<&MpsType, RuntimeError>;

    fn get_mut(&mut self, name: &str, op: &mut OpGetter) -> Result<&mut MpsType, RuntimeError>;

    fn assign(&mut self, name: &str, value: MpsType, op: &mut OpGetter) -> Result<(), RuntimeError>;

    fn declare(&mut self, name: &str, value: MpsType, op: &mut OpGetter) -> Result<(), RuntimeError>;

    fn remove(&mut self, name: &str, op: &mut OpGetter) -> Result<MpsType, RuntimeError>;
}

#[derive(Default, Debug)]
pub struct MpsOpStorage {
    storage: HashMap<String, MpsType>,
}

impl MpsVariableStorer for MpsOpStorage {
    fn get(&self, key: &str, op: &mut OpGetter) -> Result<&MpsType, RuntimeError> {
        match self.storage.get(key) {
            Some(item) => Ok(item),
            None => Err(RuntimeError {
                line: 0,
                op: op(),
                msg: format!("Variable {} not found", key),
            })
        }
    }

    fn get_mut(&mut self, key: &str, op: &mut OpGetter) -> Result<&mut MpsType, RuntimeError> {
        match self.storage.get_mut(key) {
            Some(item) => Ok(item),
            None => Err(RuntimeError {
                line: 0,
                op: op(),
                msg: format!("Variable {} not found", key),
            })
        }
    }

    fn assign(&mut self, key: &str, item: MpsType, op: &mut OpGetter) -> Result<(), RuntimeError> {
        if !self.storage.contains_key(key) {
            Err(RuntimeError {
                line: 0,
                op: op(),
                msg: format!("Cannot assign to non-existent variable {}", key),
            })
        } else {
            self.storage.insert(key.to_string(), item);
            Ok(())
        }
    }

    fn declare(&mut self, key: &str, item: MpsType, op: &mut OpGetter) -> Result<(), RuntimeError> {
        if self.storage.contains_key(key) {
            Err(RuntimeError {
                line: 0,
                op: op(),
                msg: format!("Cannot overwrite existing variable {}", key),
            })
        } else {
            self.storage.insert(key.to_string(), item);
            Ok(())
        }
    }

    fn remove(&mut self, key: &str, op: &mut OpGetter) -> Result<MpsType, RuntimeError> {
        if self.storage.contains_key(key) {
            Ok(self.storage.remove(key).unwrap())
        } else {
            Err(RuntimeError {
                line: 0,
                op: op(),
                msg: format!("Cannot remove non-existing variable {}", key),
            })
        }
    }
}