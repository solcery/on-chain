
use crate::card::{Card, CardType};

use borsh::{BorshDeserialize, BorshSerialize};

use thiserror::Error;

#[derive(Debug, Eq, PartialEq)]
pub struct CardTypesRom<'a> {
    card_types: &'a [CardType],
}

impl<'a> CardTypesRom<'a> {
    #[must_use]
    pub fn card_type_count(&self) -> usize {
        self.card_types.len()
    }

    #[must_use]
    pub fn card_type_by_type_index(&self, type_index: usize) -> &CardType {
        &self.card_types[type_index]
    }

    #[must_use]
    pub fn card_type_by_type_id(&self, type_id: u32) -> Option<&CardType> {
        self.card_types
            .iter()
            .find(|card_type| card_type.id() == type_id)
    }

    pub fn instance_card_by_type_id(&self, type_id: u32, card_id: u32) -> Result<Card, Error> {
        let typ = &self.card_types.iter().find(|typ| typ.id() == type_id);
        match typ {
            Some(typ) => Ok(typ.instantiate_card(card_id)),
            None => Err(Error::NoSuchTypeId(type_id)),
        }
    }

    pub fn instance_card_by_type_index(
        &self,
        type_index: u32,
        card_id: u32,
    ) -> Result<Card, Error> {
        let index: usize = type_index as usize;
        if index < self.card_types.len() {
            let typ: &CardType = &self.card_types[index];
            Ok(typ.instantiate_card(card_id))
        } else {
            Err(Error::TypeIndexOutOfBounds {
                index: type_index,
                len: self.card_types.len() as u32,
            })
        }
    }
    #[must_use]
    pub unsafe fn from_raw_parts(card_types: &'a [CardType]) -> Self {
        Self { card_types }
    }
}

#[derive(Error, Debug, Clone, Copy, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum Error {
    #[error("type index is out of bounds (index is {index}, but there are only {len} types)")]
    TypeIndexOutOfBounds { index: u32, len: u32 },
    #[error("there are no such type id ({0})")]
    NoSuchTypeId(u32),
}
