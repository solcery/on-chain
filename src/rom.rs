use crate::board::Board;
use crate::card::{Card, CardType};
use crate::vm::VMCommand;
use crate::word::Word;
use std::convert::TryInto;
use tinyvec::SliceVec;

const ROM_SIZE: usize = 2_usize.pow(16);
type InstructionRom<'a> = SliceVec<'a, VMCommand>;

const TYPE_DECK_SIZE: usize = 2_usize.pow(10);
type TypeDeck<'a> = SliceVec<'a, CardType>;

#[derive(Debug, Eq, PartialEq)]
pub struct Rom<'a> {
    card_types: TypeDeck<'a>,
    instructions: InstructionRom<'a>,
    initial_board_state: Board<'a>,
}
impl<'a, 'b> Rom<'a> {
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

    pub fn initialize_board(
        &self,
        deck_slice: &'b mut [Card],
        attr_slice: &'b mut [Word],
    ) -> Board<'b> {
        self.initial_board_state
            .clone_to_slices(deck_slice, attr_slice)
    }

    #[cfg(test)]
    pub unsafe fn from_raw_parts(
        instructions: &'a mut [VMCommand],
        card_types: &'a mut [CardType],
        initial_board_state: Board<'a>,
    ) -> Self {
        Rom {
            card_types: TypeDeck::from(card_types),
            instructions: InstructionRom::from(instructions),
            initial_board_state,
        }
    }
}
