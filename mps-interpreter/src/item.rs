use std::collections::HashMap;
use std::fmt::{Debug, Display, Error, Formatter};

use crate::lang::MpsTypePrimitive;
//use crate::lang::RuntimeError;
//use crate::processing::OpGetter;

/// general type object for MPS
#[derive(Clone, Debug, Default)]
pub struct MpsItem {
    fields: HashMap<String, MpsTypePrimitive>,
}

impl MpsItem {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn field(&self, name: &str) -> Option<&MpsTypePrimitive> {
        self.fields.get(name)
    }

    pub fn set_field(&mut self, name: &str, value: MpsTypePrimitive) -> Option<MpsTypePrimitive> {
        self.fields.insert(name.to_owned(), value)
    }

    pub fn set_field_chain(&mut self, name: &str, value: MpsTypePrimitive) -> &mut Self {
        self.set_field(name, value);
        self
    }

    pub fn remove_field(&mut self, name: &str) -> Option<MpsTypePrimitive> {
        self.fields.remove(name)
    }
}

impl Display for MpsItem {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(f, "MpsItem[({} fields)]", self.fields.len())
    }
}

/*pub(crate) trait MpsItemRuntimeUtil {
    fn get_field_runtime(&self, name: &str, op: &mut OpGetter) -> Result<&MpsTypePrimitive, RuntimeError>;
}

impl MpsItemRuntimeUtil for MpsItem {
    fn get_field_runtime(&self, name: &str, op: &mut OpGetter) -> Result<&MpsTypePrimitive, RuntimeError> {
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
