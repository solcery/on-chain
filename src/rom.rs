use crate::board::Board;
use crate::card::{Card, CardType};
use crate::vmcommand::VMCommand;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use tinyvec::ArrayVec;

const ROM_SIZE: usize = 2_usize.pow(5);
type InstructionRom = ArrayVec<[VMCommand; ROM_SIZE]>;

const TYPE_DECK_SIZE: usize = 2_usize.pow(5);
type TypeDeck = ArrayVec<[CardType; TYPE_DECK_SIZE]>;

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Rom {
    card_types: TypeDeck,
    instructions: InstructionRom,
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
        let mut rom = Rom {
            card_types: TypeDeck::new(),
            instructions: InstructionRom::new(),
            initial_board_state,
        };
        rom.card_types.fill(card_types);
        rom.instructions.fill(instructions);
        rom
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use flexbuffers::FlexbufferSerializer;

    #[test]
    fn serialize() {
        let instructions = vec![VMCommand::Halt];
        let rom = unsafe { Rom::from_raw_parts(instructions, vec![], Board::new()) };
        let mut ser = FlexbufferSerializer::new();
        rom.serialize(&mut ser).unwrap();

        let data = ser.view();

        let r = flexbuffers::Reader::get_root(data).unwrap();

        let rom2 = Rom::deserialize(r).unwrap();

        assert_eq!(rom, rom2);
    }
}
