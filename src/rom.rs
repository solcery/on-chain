use crate::board::Board;
use crate::card::{Card, CardType};
use crate::vmcommand::VMCommand;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Rom {
    card_types: Vec<CardType>,
    instructions: Vec<VMCommand>,
    initial_board_state: Board,
}
impl Rom {
    pub fn fetch_instruction(&self, pc: usize) -> VMCommand {
        self.instructions[pc]
    }

    pub fn card_type_count(&self) -> usize {
        self.card_types.len()
    }

    pub fn card_type_by_type_index(&self, type_index: usize) -> &CardType {
        &self.card_types[type_index]
    }

    pub fn card_type_by_type_id(&self, type_id: u32) -> Option<&CardType> {
        self.card_types
            .iter()
            .find(|card_type| card_type.id() == type_id)
    }

    pub fn instance_card_by_type_id(&self, type_id: u32, card_id: u32) -> Result<Card, ()> {
        let typ = &self.card_types.iter().find(|typ| typ.id() == type_id);
        match typ {
            Some(typ) => Ok(typ.instantiate_card(card_id)),
            None => Err(()),
        }
    }

    pub fn instance_card_by_type_index(&self, type_index: u32, card_id: u32) -> Result<Card, ()> {
        let index: usize = type_index.try_into().unwrap();
        if index < self.card_types.len() {
            let typ: &CardType = &self.card_types[index];
            Ok(typ.instantiate_card(card_id))
        } else {
            Err(())
        }
    }

    pub fn initialize_board(&self) -> Board {
        self.initial_board_state.clone()
    }

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
