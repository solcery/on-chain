//! # The Sorcery Virtual Machine

use crate::board::Board;
use crate::rom::Rom;
use crate::word::Word;
use std::convert::TryInto;

mod memory;
use memory::Memory;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VMCommand {
    // Arithmetic
    /// Adds two topmost values from the stack
    /// # Panics
    /// Panics if there are no enough elements on the stack or if one of the arguments is
    /// [Word::Boolean]
    Add,
    Sub,
    Div,
    /// Multiplies two topmost values from the stack
    /// # Panics
    /// Panics if there are no enough elements on the stack or if one of the arguments is
    /// [Word::Boolean]
    Mul,
    Rem,
    /// Negates the topmost value on the stack
    /// # Panics
    /// Panics if there are no enough elements on the stack or if  the argumentsis
    /// [Word::Boolean]
    Neg,
    /// Increments the topmost value on the stack
    /// Panics if there are no enough elements on the stack or if  the argumentsis
    /// [Word::Boolean]
    Inc,
    /// Decrements the topmost value on the stack
    /// Panics if there are no enough elements on the stack or if  the argumentsis
    /// [Word::Boolean]
    Dec,
    /// Computes the absolute value of the topmost value on the stack
    /// Panics if there are no enough elements on the stack or if  the argumentsis
    /// [Word::Boolean]
    Abs,

    // Logic
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,

    // Data transfer
    PushConstant(Word),
    PushBoardAttr {
        index: usize,
    },
    PopBoardAttr {
        index: usize,
    },
    PushLocal {
        index: usize,
    },
    PopLocal {
        index: usize,
    },
    PushArgument {
        index: usize,
    },
    PopArgument {
        index: usize,
    },

    // Flow control
    Goto(usize),
    IfGoto(usize),
    Halt,
    Function {
        n_locals: usize,
    },
    Call {
        address: usize,
        n_args: usize,
    },
    Return,

    // Interactions with cards
    /// Pushes total number of cards to the stack
    PushCardCount,
    /// Pushes total number of card types to the stack
    PushTypeCount,
    /// Pushes [CardType](crate::card::CardType) on the `i`-th card, where `i` is the topmost value on the stack
    PushCardType,
    /// Pushes total number of cards with [CardType](crate::card::CardType) popped from the stack
    PushCardCountWithCardType,
    /// Pushes `attr_index`-th attribute of the [CardType](crate::card::CardType), those index
    /// among [CardTypes](crate::card::CardType) is on the top of the stack
    PushCardTypeAttrByTypeIndex {
        attr_index: usize,
    },
    /// Pushes `attr_index`-th attribute of the [CardType](crate::card::CardType) of the card,
    /// those index is on the top of the stack
    PushCardTypeAttrByCardIndex {
        attr_index: usize,
    },
    /// Pushes `attr_index`-th attribute of the [Card](crate::card::Card),
    /// those index is on the top of the stack
    PushCardAttr {
        attr_index: usize,
    },
    /// Pops `attr_index`-th attribute of the [Card](crate::card::Card),
    /// those index is on the top of the stack
    PopCardAttr {
        attr_index: usize,
    },

    /// Pops [CardType](crate::card::CardType) index from the stack and calls it's `action_id` action as a function
    InstanceCardByTypeIndex,
    /// Pops [CardType](crate::card::CardType) id from the stack and calls it's `action_id` action as a function
    InstanceCardByTypeId,
    CallCardAction {
        action_id: usize,
    },
    RemoveCardByIndex {
        card_index: usize,
    },
}

impl Default for VMCommand {
    fn default() -> Self {
        VMCommand::Halt
    }
}

pub struct VM<'a> {
    rom: &'a Rom,
    memory: Memory,
    board: &'a mut Board,
}

impl<'a> VM<'a> {
    pub fn new(rom: &'a Rom, board: &'a mut Board) -> VM<'a> {
        VM {
            rom,
            memory: Memory::new(),
            board,
        }
    }

    pub fn execute(&mut self, instruction_limit: usize) {
        for _ in 0..instruction_limit {
            if self.run_one_instruction().is_err() {
                break;
            }
        }
    }

    fn run_one_instruction(&mut self) -> Result<(), ()> {
        //TODO: better handing for Halt instruction.
        //Probably, we need to propogate errors from the instructions to this function.
        let instruction = self.rom.fetch_instruction(self.memory.pc());
        match instruction {
            VMCommand::Add => {
                self.memory.add();
                Ok(())
            }
            VMCommand::Sub => {
                self.memory.sub();
                Ok(())
            }
            VMCommand::Mul => {
                self.memory.mul();
                Ok(())
            }
            VMCommand::Div => {
                self.memory.div();
                Ok(())
            }
            VMCommand::Rem => {
                self.memory.rem();
                Ok(())
            }
            VMCommand::Neg => {
                self.memory.neg();
                Ok(())
            }
            VMCommand::Inc => {
                self.memory.inc();
                Ok(())
            }
            VMCommand::Dec => {
                self.memory.dec();
                Ok(())
            }
            VMCommand::Abs => {
                self.memory.abs();
                Ok(())
            }
            VMCommand::Eq => {
                self.memory.eq();
                Ok(())
            }
            VMCommand::Gt => {
                self.memory.gt();
                Ok(())
            }
            VMCommand::Lt => {
                self.memory.lt();
                Ok(())
            }
            VMCommand::Or => {
                self.memory.or();
                Ok(())
            }
            VMCommand::And => {
                self.memory.and();
                Ok(())
            }
            VMCommand::Not => {
                self.memory.not();
                Ok(())
            }
            VMCommand::PushConstant(word) => {
                self.memory.push_external(word);
                Ok(())
            }
            VMCommand::PushBoardAttr { index } => {
                let attr = self.board.attrs[index];
                self.memory.push_external(attr);
                Ok(())
            }
            VMCommand::PopBoardAttr { index } => {
                let value = self.memory.pop_external();
                self.board.attrs[index] = value;
                Ok(())
            }
            VMCommand::PushLocal { index } => {
                self.memory.push_local(index);
                Ok(())
            }
            VMCommand::PopLocal { index } => {
                self.memory.pop_local(index);
                Ok(())
            }
            VMCommand::PushArgument { index } => {
                self.memory.push_argument(index);
                Ok(())
            }
            VMCommand::PopArgument { index } => {
                self.memory.pop_argument(index);
                Ok(())
            }
            VMCommand::Goto(instruction) => {
                self.memory.jmp(instruction);
                Ok(())
            }
            VMCommand::IfGoto(instruction) => {
                self.memory.ifjmp(instruction);
                Ok(())
            }
            VMCommand::Call { address, n_args } => {
                self.memory.call(address, n_args);
                Ok(())
            }
            VMCommand::Function { n_locals } => {
                self.memory.function(n_locals);
                Ok(())
            }
            VMCommand::Return => {
                self.memory.fn_return();
                Ok(())
            }
            VMCommand::PushCardCount => {
                let len = self.board.cards.len();
                self.memory
                    .push_external(Word::Numeric(TryInto::try_into(len).unwrap()));
                Ok(())
            }
            VMCommand::PushTypeCount => {
                let len = self.rom.card_type_count();
                self.memory
                    .push_external(Word::Numeric(TryInto::try_into(len).unwrap()));
                Ok(())
            }
            VMCommand::PushCardCountWithCardType => {
                self.push_card_count_with_type();
                Ok(())
            }
            VMCommand::PushCardType => {
                self.push_card_type();
                Ok(())
            }
            VMCommand::PushCardTypeAttrByTypeIndex { attr_index } => {
                self.push_card_type_attr_by_type_index(attr_index);
                Ok(())
            }
            VMCommand::PushCardTypeAttrByCardIndex { attr_index } => {
                self.push_card_type_attr_by_card_index(attr_index);
                Ok(())
            }
            VMCommand::PushCardAttr { attr_index } => {
                self.push_card_attr(attr_index);
                Ok(())
            }
            VMCommand::PopCardAttr { attr_index } => {
                self.pop_card_attr(attr_index);
                Ok(())
            }
            VMCommand::InstanceCardByTypeIndex => {
                self.instantiate_card_by_type_index();
                Ok(())
            }
            VMCommand::InstanceCardByTypeId => {
                self.instantiate_card_by_type_id();
                Ok(())
            }
            VMCommand::Halt => Err(()),
            _ => {
                unimplemented!();
            }
        }
    }

    fn push_card_type(&mut self) {
        let index = self.memory.pop_external_no_pc_inc();
        match index {
            Word::Numeric(i) => {
                let card_type = self.board.cards[i as usize].card_type();
                let word = Word::Numeric(TryInto::try_into(card_type).unwrap());
                self.memory.push_external(word);
            }
            Word::Boolean(_) => {
                panic!("Type mismath: bool can not be interpreted as index.");
            }
        }
    }

    fn push_card_count_with_type(&mut self) {
        let card_type = self.memory.pop_external_no_pc_inc();
        match card_type {
            Word::Numeric(id) => {
                // Word::Numeric contains i32, but card_type is u32, so convert is needed
                let signed_card_type = id.try_into().unwrap();
                let count = self
                    .board
                    .cards
                    .iter()
                    .filter(|card| card.card_type() == signed_card_type)
                    .count();

                let word = Word::Numeric(TryInto::try_into(count).unwrap());
                self.memory.push_external(word);
            }
            Word::Boolean(_) => {
                panic!("Type mismath: bool can not be interpreted as index.");
            }
        }
    }

    fn push_card_type_attr_by_type_index(&mut self, attr_index: usize) {
        let type_index = self.memory.pop_external_no_pc_inc();
        match type_index {
            Word::Numeric(id) => {
                let card_type = self.rom.card_type_by_type_index(id as usize);
                let attr_value = card_type.attr_by_index(attr_index);

                let word = attr_value;
                self.memory.push_external(word);
            }
            Word::Boolean(_) => {
                panic!("Type mismath: bool can not be interpreted as index.");
            }
        }
    }

    fn push_card_type_attr_by_card_index(&mut self, attr_index: usize) {
        let card_index = self.memory.pop_external_no_pc_inc();
        match card_index {
            Word::Numeric(id) => {
                let card = &self.board.cards[id as usize];
                let card_type_id = card.card_type();
                let card_type = self.rom.card_type_by_type_id(card_type_id).unwrap();
                let attr_value = card_type.attr_by_index(attr_index);

                let word = attr_value;
                self.memory.push_external(word);
            }
            Word::Boolean(_) => {
                panic!("Type mismath: bool can not be interpreted as index.");
            }
        }
    }

    fn push_card_attr(&mut self, attr_index: usize) {
        let card_index = self.memory.pop_external_no_pc_inc();
        match card_index {
            Word::Numeric(id) => {
                let card = &self.board.cards[id as usize];
                let attr_value = card.attrs[attr_index];

                let word = attr_value;
                self.memory.push_external(word);
            }
            Word::Boolean(_) => {
                panic!("Type mismath: bool can not be interpreted as index.");
            }
        }
    }

    fn pop_card_attr(&mut self, attr_index: usize) {
        let card_index = self.memory.pop_external_no_pc_inc();
        match card_index {
            Word::Numeric(id) => {
                let card = &mut self.board.cards[id as usize];
                let attr_value = self.memory.pop_external();

                card.attrs[attr_index] = attr_value;
            }
            Word::Boolean(_) => {
                panic!("Type mismath: bool can not be interpreted as index.");
            }
        }
    }

    fn instantiate_card_by_type_index(&mut self) {
        let index = self.memory.pop_external();
        match index {
            Word::Numeric(index) => {
                let id = index.try_into().unwrap();
                let card = self.rom.instance_card_by_type_index(id, self.board.generate_card_id()).unwrap();
                self.board.cards.push(card);
            }
            Word::Boolean(_) => {
                panic!("Type mismath: bool can not be interpreted as index.");
            }
        }
        
    }

    fn instantiate_card_by_type_id(&mut self) {
        let index = self.memory.pop_external();
        match index {
            Word::Numeric(index) => {
                let id = index.try_into().unwrap();
                let card = self.rom.instance_card_by_type_id(id, self.board.generate_card_id()).unwrap();
                self.board.cards.push(card);
            }
            Word::Boolean(_) => {
                panic!("Type mismath: bool can not be interpreted as index.");
            }
        }
        
    }

    #[cfg(test)]
    unsafe fn from_raw_parts(rom: &'a Rom, memory: Memory, board: &'a mut Board) -> VM<'a> {
        VM { rom, memory, board }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{CardType, EntryPoint};
    use crate::word_vec;

    fn testing_board() -> Board {
        let type1_attrs = word_vec![10, 5, true, false,];
        let type1_init_attrs = word_vec![5, 5, false, false,];
        let type1 = unsafe {
            CardType::from_raw_parts(
                1,
                type1_attrs,
                type1_init_attrs,
                vec![EntryPoint::from_raw_parts(1, 0)],
            )
        };

        let type2_attrs = word_vec![20, 5, true, true,];
        let type2_init_attrs = word_vec![6, 4, false, false,];
        let type2 = unsafe {
            CardType::from_raw_parts(
                2,
                type2_attrs,
                type2_init_attrs,
                vec![EntryPoint::from_raw_parts(4, 0)],
            )
        };

        let board_attrs = word_vec![3, 4, 5, false, false, true,];

        let mut card1 = type1.instantiate_card(1);
        let mut card2 = type2.instantiate_card(2);

        card1.attrs[0] = Word::Numeric(4);
        card2.attrs[3] = Word::Boolean(true);

        let card3 = type1.instantiate_card(3);
        let card4 = type2.instantiate_card(4);

        unsafe { Board::from_raw_parts(vec![card1, card2, card3, card4], board_attrs, 5) }
    }
}
