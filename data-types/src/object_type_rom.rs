use crate::object::Object;
use crate::object_type::ObjectType;

use borsh::{BorshDeserialize, BorshSerialize};

use thiserror::Error;

#[derive(Debug, Eq, PartialEq)]
pub struct ObjectTypesRom<'a> {
    object_types: &'a [ObjectType],
}

impl<'a> ObjectTypesRom<'a> {
    #[must_use]
    pub fn object_type_count(&self) -> usize {
        self.object_types.len()
    }

    #[must_use]
    pub fn object_type_by_type_index(&self, type_index: usize) -> &ObjectType {
        &self.object_types[type_index]
    }

    #[must_use]
    pub fn object_type_by_type_id(&self, type_id: u32) -> Option<&ObjectType> {
        self.object_types
            .iter()
            .find(|object_type| object_type.id() == type_id)
    }

    pub fn instance_object_by_type_id(
        &self,
        type_id: u32,
        object_id: u32,
    ) -> Result<Object, Error> {
        let typ = &self.object_types.iter().find(|typ| typ.id() == type_id);
        match typ {
            Some(typ) => Ok(typ.instantiate_object(object_id)),
            None => Err(Error::NoSuchTypeId(type_id)),
        }
    }

    pub fn instance_object_by_type_index(
        &self,
        type_index: u32,
        object_id: u32,
    ) -> Result<Object, Error> {
        let index: usize = type_index as usize;
        if index < self.object_types.len() {
            let typ: &ObjectType = &self.object_types[index];
            Ok(typ.instantiate_object(object_id))
        } else {
            Err(Error::TypeIndexOutOfBounds {
                index: type_index,
                len: self.object_types.len() as u32,
            })
        }
    }
    #[must_use]
    pub unsafe fn from_raw_parts(object_types: &'a [ObjectType]) -> Self {
        Self { object_types }
    }
}

#[derive(Error, Debug, Clone, Copy, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum Error {
    #[error("type index is out of bounds (index is {index}, but there are only {len} types)")]
    TypeIndexOutOfBounds { index: u32, len: u32 },
    #[error("there are no such type id ({0})")]
    NoSuchTypeId(u32),
}
