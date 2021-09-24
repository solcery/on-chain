use crate::card::{Card, CardType};
use crate::board::Board;
use crate::vm::VMCommand;
use std::convert::TryInto;
use tinyvec::ArrayVec;

const ROM_SIZE: usize = 2 ^ 16;
type InstructionRom = ArrayVec<[VMCommand; ROM_SIZE]>;

const TYPE_DECK_SIZE: usize = 2 ^ 10;
type TypeDeck = ArrayVec<[CardType; TYPE_DECK_SIZE]>;

pub struct Rom {
    card_types: TypeDeck,
    rom: InstructionRom,
    initial_board_state: Board,
}
impl Rom {
    pub fn fetch_instruction(&self, pc: usize) -> VMCommand {
        self.rom[pc]
    }

    pub fn add_type(&mut self, typ: CardType) {
        self.card_types.push(typ);
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
        let index = type_index.try_into().unwrap();
        if self.card_types.len() < index {
            let typ = &self.card_types[index];
            Ok(typ.instantiate_card(card_id))
        } else {
            Err(())
        }
    }

    pub fn initialize_board(&self) -> Board {
        self.initial_board_state.clone()
    }
}
