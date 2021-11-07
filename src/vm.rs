//! # The Sorcery Virtual Machine

use crate::board::Board;
use crate::rom::Rom;
use crate::vmcommand::VMCommand;
use crate::word::Word;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;

mod memory;
use memory::Memory;
use memory::VMError;

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Sealed<T> {
    data: T,
}

impl<T> Sealed<T> {
    fn release_data(self) -> T {
        self.data
    }
}

pub struct VM<'a> {
    rom: &'a Rom,
    memory: Memory,
    board: &'a mut Board,
}

impl<'a> VM<'a> {
    pub fn init_vm(
        rom: &'a Rom,
        board: &'a mut Board,
        args: &'a [Word],
        card_index: u32,
        action_index: u32,
    ) -> Self {
        let memory = Memory::init_memory(args, card_index, action_index);
        Self { rom, memory, board }
    }

    pub fn execute(&mut self, instruction_limit: usize) {
        for _ in 0..instruction_limit {
            if self.run_one_instruction().is_err() {
                break;
            }
        }
    }

    pub fn resume_execution(
        rom: &'a Rom,
        board: &'a mut Board,
        sealed_memory: Sealed<Memory>,
    ) -> Self {
        let memory = Sealed::<Memory>::release_data(sealed_memory);
        Self { rom, memory, board }
    }

    #[must_use]
    pub fn stop_execution(self) -> Sealed<Memory> {
        Sealed::<Memory> { data: self.memory }
    }

    fn run_one_instruction(&mut self) -> Result<(), VMError> {
        //TODO: better handing for Halt instruction.
        //Probably, we need to propogate errors from the instructions to this function.
        let instruction = self.rom.fetch_instruction(self.memory.pc());
        match instruction {
            VMCommand::Add => self.memory.add(),
            VMCommand::Sub => self.memory.sub(),
            VMCommand::Mul => self.memory.mul(),
            VMCommand::Div => self.memory.div(),
            VMCommand::Rem => self.memory.rem(),
            VMCommand::Neg => self.memory.neg(),
            VMCommand::Inc => self.memory.inc(),
            VMCommand::Dec => self.memory.dec(),
            VMCommand::Abs => self.memory.abs(),
            VMCommand::Eq => self.memory.equal(),
            VMCommand::Gt => self.memory.gt(),
            VMCommand::Lt => self.memory.lt(),
            VMCommand::Or => self.memory.or(),
            VMCommand::And => self.memory.and(),
            VMCommand::Not => self.memory.not(),
            VMCommand::PushConstant(word) => self.memory.push_external(word),
            VMCommand::PushBoardAttr { index } => {
                let attr = self.board.attrs[index as usize];
                self.memory.push_external(attr)
            }
            VMCommand::PopBoardAttr { index } => {
                let value = self.memory.pop_external()?;
                self.board.attrs[index as usize] = value;
                Ok(())
            }
            VMCommand::PushLocal { index } => self.memory.push_local(index as usize),
            VMCommand::PopLocal { index } => self.memory.pop_local(index as usize),
            VMCommand::PushArgument { index } => self.memory.push_argument(index as usize),
            VMCommand::PopArgument { index } => self.memory.pop_argument(index as usize),
            VMCommand::Goto(instruction) => self.memory.jmp(instruction as usize),
            VMCommand::IfGoto(instruction) => self.memory.ifjmp(instruction as usize),
            VMCommand::Call { address, n_args } => {
                self.memory.call(address as usize, n_args as usize)
            }
            VMCommand::Function { n_locals } => self.memory.function(n_locals as usize),
            VMCommand::Return => self.memory.fn_return(),
            VMCommand::ReturnVoid => self.memory.return_void(),
            VMCommand::PushCardCount => {
                let len = self.board.cards.len();
                self.memory.push_external(Word::Numeric(len as i32))
            }
            VMCommand::PushTypeCount => {
                let len = self.rom.card_type_count();
                self.memory.push_external(Word::Numeric(len as i32))
            }
            VMCommand::PushCardCountWithCardType => self.push_card_count_with_type(),
            VMCommand::PushCardType => self.push_card_type(),
            VMCommand::PushCardTypeAttrByTypeIndex { attr_index } => {
                self.push_card_type_attr_by_type_index(attr_index)
            }
            VMCommand::PushCardTypeAttrByCardIndex { attr_index } => {
                self.push_card_type_attr_by_card_index(attr_index)
            }
            VMCommand::PushCardAttr { attr_index } => self.push_card_attr(attr_index),
            VMCommand::PopCardAttr { attr_index } => self.pop_card_attr(attr_index),
            VMCommand::InstanceCardByTypeIndex => self.instantiate_card_by_type_index(),
            VMCommand::InstanceCardByTypeId => self.instantiate_card_by_type_id(),
            VMCommand::CallCardAction => self.call_card_action(),
            VMCommand::RemoveCardByIndex => self.remove_card_by_index(),
            VMCommand::Halt => Err(VMError::Halt),
        }
    }

    fn push_card_type(&mut self) -> Result<(), VMError> {
        let index = self.memory.pop_external_no_pc_inc()?;
        match index {
            Word::Numeric(i) => {
                let card_type = self.board.cards[i as usize].card_type();
                let word = Word::Numeric(TryInto::try_into(card_type).unwrap());
                self.memory.push_external(word)
            }
            Word::Boolean(_) => Err(VMError::TypeMismatch),
        }
    }

    fn push_card_count_with_type(&mut self) -> Result<(), VMError> {
        let card_type = self.memory.pop_external_no_pc_inc()?;
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
                self.memory.push_external(word)
            }
            Word::Boolean(_) => Err(VMError::TypeMismatch),
        }
    }

    fn push_card_type_attr_by_type_index(&mut self, attr_index: u32) -> Result<(), VMError> {
        let type_index = self.memory.pop_external_no_pc_inc()?;
        match type_index {
            Word::Numeric(id) => {
                let card_type = self.rom.card_type_by_type_index(id as usize);
                let attr_value = card_type.attr_by_index(attr_index as usize);

                let word = attr_value;
                self.memory.push_external(word)
            }
            Word::Boolean(_) => Err(VMError::TypeMismatch),
        }
    }

    fn push_card_type_attr_by_card_index(&mut self, attr_index: u32) -> Result<(), VMError> {
        let card_index = self.memory.pop_external_no_pc_inc()?;
        match card_index {
            Word::Numeric(id) => {
                let card = &self.board.cards[id as usize];
                let card_type_id = card.card_type();
                let card_type = self.rom.card_type_by_type_id(card_type_id).unwrap();
                let attr_value = card_type.attr_by_index(attr_index as usize);

                let word = attr_value;
                self.memory.push_external(word)
            }
            Word::Boolean(_) => Err(VMError::TypeMismatch),
        }
    }

    fn push_card_attr(&mut self, attr_index: u32) -> Result<(), VMError> {
        let card_index = self.memory.pop_external_no_pc_inc()?;
        match card_index {
            Word::Numeric(id) => {
                let card = &self.board.cards[id as usize];
                let attr_value = card.attrs[attr_index as usize];

                let word = attr_value;
                self.memory.push_external(word)
            }
            Word::Boolean(_) => {
                panic!("Type mismath: bool can not be interpreted as index.");
            }
        }
    }

    fn pop_card_attr(&mut self, attr_index: u32) -> Result<(), VMError> {
        let card_index = self.memory.pop_external_no_pc_inc()?;
        match card_index {
            Word::Numeric(id) => {
                let card = &mut self.board.cards[id as usize];
                let attr_value = self.memory.pop_external()?;

                card.attrs[attr_index as usize] = attr_value;
                Ok(())
            }
            Word::Boolean(_) => Err(VMError::TypeMismatch),
        }
    }

    fn instantiate_card_by_type_index(&mut self) -> Result<(), VMError> {
        let index = self.memory.pop_external()?;
        match index {
            Word::Numeric(index) => {
                let id = index.try_into().unwrap();
                let card = self
                    .rom
                    .instance_card_by_type_index(id, self.board.generate_card_id())
                    .unwrap();
                self.board.cards.push(card);
                Ok(())
            }
            Word::Boolean(_) => Err(VMError::TypeMismatch),
        }
    }

    fn instantiate_card_by_type_id(&mut self) -> Result<(), VMError> {
        let index = self.memory.pop_external()?;
        match index {
            Word::Numeric(index) => {
                let id = index.try_into().unwrap();
                let card = self
                    .rom
                    .instance_card_by_type_id(id, self.board.generate_card_id())
                    .unwrap();
                self.board.cards.push(card);
                Ok(())
            }
            Word::Boolean(_) => Err(VMError::TypeMismatch),
        }
    }

    fn call_card_action(&mut self) -> Result<(), VMError> {
        let action_index_word = self.memory.pop_external_no_pc_inc()?;
        let action_index = i32::try_from(action_index_word).map_err(|_| VMError::TypeMismatch)?;

        let type_index_word = self.memory.pop_external_no_pc_inc()?;
        let type_index = i32::try_from(type_index_word).map_err(|_| VMError::TypeMismatch)?;

        let card_type = self.rom.card_type_by_type_index(type_index as usize);
        let entry_point = card_type.action_entry_point(action_index as usize);
        self.memory
            .call(entry_point.address(), entry_point.n_args())
    }

    fn remove_card_by_index(&mut self) -> Result<(), VMError> {
        let card_index_word = self.memory.pop_external()?;
        let card_index = i32::try_from(card_index_word).map_err(|_| VMError::TypeMismatch)?;

        self.board.cards.remove(card_index as usize);
        Ok(())
    }

    #[cfg(test)]
    fn release_memory(self) -> Memory {
        self.memory
    }

    #[must_use]
    pub fn is_halted(&self) -> bool {
        let instruction = self.rom.fetch_instruction(self.memory.pc());
        instruction == VMCommand::Halt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{CardType, EntryPoint};
    use crate::word_vec;
    use pretty_assertions::assert_eq;

    fn type1() -> CardType {
        let type1_attrs = word_vec![10, 5, true, false,];
        let type1_init_attrs = word_vec![5, 5, false, false,];
        unsafe {
            CardType::from_raw_parts(
                1,
                type1_attrs,
                type1_init_attrs,
                vec![EntryPoint::from_raw_parts(2, 0)],
            )
        }
    }

    fn type2() -> CardType {
        let type2_attrs = word_vec![20, 5, true, true,];
        let type2_init_attrs = word_vec![6, 4, false, false,];
        unsafe {
            CardType::from_raw_parts(
                2,
                type2_attrs,
                type2_init_attrs,
                vec![EntryPoint::from_raw_parts(4, 0)],
            )
        }
    }

    fn testing_board() -> Board {
        let type1 = type1();
        let type2 = type2();

        let board_attrs = word_vec![3, 4, 5, false, false, true,];

        let mut card1 = type1.instantiate_card(1);
        let mut card2 = type2.instantiate_card(2);

        card1.attrs[0] = Word::Numeric(4);
        card2.attrs[3] = Word::Boolean(true);

        let card3 = type1.instantiate_card(3);
        let card4 = type2.instantiate_card(4);

        unsafe { Board::from_raw_parts(vec![card1, card2, card3, card4], board_attrs, 5) }
    }

    fn initial_board() -> Board {
        let card1 = type1().instantiate_card(1);
        let board_attrs = word_vec![0, 0, 0, false, false, false,];
        unsafe { Board::from_raw_parts(vec![card1], board_attrs, 1) }
    }

    #[test]
    fn init_empty_memory_vm() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::PushCardType,
            VMCommand::Halt,
        ];
        let card_types = vec![type1(), type2()];

        let rom = unsafe { Rom::from_raw_parts(instructions, card_types, initial_board()) };
        let mut board = testing_board();

        let args = vec![];
        let vm = VM::init_vm(&rom, &mut board, &args, 0, 0);
        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 0, 0, 0) };

        assert_eq!(memory, needed_memory);
    }

    #[test]
    fn push_type() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::PushCardType,
            VMCommand::Halt,
        ];
        let card_types = vec![type1(), type2()];

        let rom = unsafe { Rom::from_raw_parts(instructions, card_types, initial_board()) };
        let mut board = testing_board();

        let args = vec![];
        let mut vm = VM::init_vm(&rom, &mut board, &args, 0, 0);

        vm.execute(10);
        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, 1], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let board_needed = testing_board();

        assert_eq!(board, board_needed);
    }

    #[test]
    fn push_card_count() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::PushCardCountWithCardType,
            VMCommand::Halt,
        ];
        let card_types = vec![type1(), type2()];

        let rom = unsafe { Rom::from_raw_parts(instructions, card_types, initial_board()) };
        let mut board = testing_board();

        let args = vec![];
        let mut vm = VM::init_vm(&rom, &mut board, &args, 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, 2], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let board_needed = testing_board();

        assert_eq!(board, board_needed);
    }

    #[test]
    fn push_type_attr_by_type_index() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(1)),
            VMCommand::PushCardTypeAttrByTypeIndex { attr_index: 3 },
            VMCommand::Halt,
        ];
        let card_types = vec![type1(), type2()];

        let rom = unsafe { Rom::from_raw_parts(instructions, card_types, initial_board()) };
        let mut board = testing_board();

        let args = vec![];
        let mut vm = VM::init_vm(&rom, &mut board, &args, 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let board_needed = testing_board();

        assert_eq!(board, board_needed);
    }

    #[test]
    fn push_type_attr_by_card_index() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(1)),
            VMCommand::PushCardTypeAttrByCardIndex { attr_index: 3 },
            VMCommand::Halt,
        ];
        let card_types = vec![type1(), type2()];

        let rom = unsafe { Rom::from_raw_parts(instructions, card_types, initial_board()) };
        let mut board = testing_board();

        let args = vec![];
        let mut vm = VM::init_vm(&rom, &mut board, &args, 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let board_needed = testing_board();

        assert_eq!(board, board_needed);
    }

    #[test]
    fn push_attr() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(1)),
            VMCommand::PushCardAttr { attr_index: 3 },
            VMCommand::Halt,
        ];
        let card_types = vec![type1(), type2()];

        let rom = unsafe { Rom::from_raw_parts(instructions, card_types, initial_board()) };
        let mut board = testing_board();

        let args = vec![];
        let mut vm = VM::init_vm(&rom, &mut board, &args, 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let board_needed = testing_board();

        assert_eq!(board, board_needed);
    }

    #[test]
    fn pop_attr() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(42)),
            VMCommand::PushConstant(Word::Numeric(1)),
            VMCommand::PopCardAttr { attr_index: 3 },
            VMCommand::Halt,
        ];
        let card_types = vec![type1(), type2()];

        let rom = unsafe { Rom::from_raw_parts(instructions, card_types, initial_board()) };
        let mut board = testing_board();

        let args = vec![];
        let mut vm = VM::init_vm(&rom, &mut board, &args, 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 3, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut board_needed = testing_board();
        board_needed.cards[1].attrs[3] = Word::Numeric(42);

        assert_eq!(board, board_needed);
    }

    #[test]
    fn add_one_card_by_index() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(1)),
            VMCommand::InstanceCardByTypeIndex,
            VMCommand::Halt,
        ];
        let card_types = vec![type1(), type2()];

        let rom = unsafe { Rom::from_raw_parts(instructions, card_types, initial_board()) };
        let mut board = initial_board();

        let args = vec![];
        let mut vm = VM::init_vm(&rom, &mut board, &args, 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut board_needed = initial_board();
        let _ = board_needed.generate_card_id();
        let added_card = type2().instantiate_card(1);
        board_needed.cards.push(added_card);

        assert_eq!(board, board_needed);
    }

    #[test]
    fn add_one_card_by_id() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::InstanceCardByTypeId,
            VMCommand::Halt,
        ];
        let card_types = vec![type1(), type2()];

        let rom = unsafe { Rom::from_raw_parts(instructions, card_types, initial_board()) };
        let mut board = initial_board();

        let args = vec![];
        let mut vm = VM::init_vm(&rom, &mut board, &args, 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut board_needed = initial_board();
        let _ = board_needed.generate_card_id();
        let added_card = type2().instantiate_card(1);
        board_needed.cards.push(added_card);

        assert_eq!(board, board_needed);
    }

    #[test]
    fn add_one_card() {
        let instructions = vec![
            VMCommand::CallCardAction,
            VMCommand::Halt,
            //{
            VMCommand::Function { n_locals: 0 }, // Добавляет на доску одну карту типа 2
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::InstanceCardByTypeId,
            VMCommand::PushConstant(Word::Numeric(5)),
            VMCommand::Return,
            //}
        ];

        let card_types = vec![type1(), type2()];

        let rom = unsafe { Rom::from_raw_parts(instructions, card_types, initial_board()) };
        let mut board = initial_board();

        let args = vec![];
        let mut vm = VM::init_vm(&rom, &mut board, &args, 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![5], 0, 0, 1, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut board_needed = initial_board();
        let card = type2().instantiate_card(board_needed.generate_card_id());
        board_needed.cards.push(card);

        assert_eq!(board, board_needed);
    }

    #[test]
    fn remove_one_card() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(0)),
            VMCommand::RemoveCardByIndex,
            VMCommand::Halt,
        ];

        let card_types = vec![type1(), type2()];

        let rom = unsafe { Rom::from_raw_parts(instructions, card_types, initial_board()) };
        let mut board = testing_board();

        let args = vec![];
        let mut vm = VM::init_vm(&rom, &mut board, &args, 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut board_needed = testing_board();
        board_needed.cards.remove(0);

        assert_eq!(board, board_needed);
    }
}
