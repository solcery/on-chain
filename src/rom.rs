use crate::board::Board;
use crate::card::{Card, CardType};
use crate::vmcommand::VMCommand;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Rom {
    card_types: Vec<CardType>,
    instructions: Vec<VMCommand>,
    initial_board_state: Board,
}

#[derive(Debug, Eq, PartialEq)]
pub struct InstructionRom<'a> {
    instructions: &'a [VMCommand],
}

impl<'a> InstructionRom<'a> {
    #[must_use]
    pub fn fetch_instruction(&self, pc: usize) -> VMCommand {
        self.instructions[pc]
    }

    #[must_use]
    pub unsafe fn from_raw_parts(instructions: &'a [VMCommand]) -> Self {
        Self { instructions }
    }
}

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

impl Rom {
    #[must_use]
    pub fn fetch_instruction(&self, pc: usize) -> VMCommand {
        self.instructions[pc]
    }

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
    pub fn initialize_board(&self) -> Board {
        self.initial_board_state.clone()
    }

    #[must_use]
    pub unsafe fn from_raw_parts(
        instructions: Vec<VMCommand>,
        card_types: Vec<CardType>,
        initial_board_state: Board,
    ) -> Self {
        Self {
            card_types,
            instructions,
            initial_board_state,
        }
    }
}

#[derive(Error, Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum Error {
    #[error("type index is out of bounds (index is {index}, but there are only {len} types)")]
    TypeIndexOutOfBounds { index: u32, len: u32 },
    #[error("there are no such type id ({0})")]
    NoSuchTypeId(u32),
}
