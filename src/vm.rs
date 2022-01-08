//! # The Sorcery Virtual Machine

use crate::board::Board;
use crate::instruction_rom::InstructionRom;
use crate::rom::CardTypesRom;
use crate::vmcommand::VMCommand;
use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryFrom;
use std::convert::TryInto;

mod memory;
use memory::Error as InternalError;
use memory::Memory;

mod log;
use log::{Event, Log};

mod enums;
use enums::ExecutionResult;
pub use enums::SingleExecutionResult;

mod error;
pub use error::Error;

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Sealed<T> {
    data: T,
}

impl<T> Sealed<T> {
    fn release_data(self) -> T {
        self.data
    }
}

#[derive(Debug)]
pub struct VM<'a> {
    instructions: InstructionRom<'a>,
    card_types: CardTypesRom<'a>,
    memory: Memory,
    board: &'a mut Board,
    log: Log,
}

impl<'a> VM<'a> {
    pub fn init_vm(
        instructions: InstructionRom<'a>,
        card_types: CardTypesRom<'a>,
        board: &'a mut Board,
        args: &'a [Word],
        card_index: u32,
        action_index: u32,
    ) -> Self {
        let memory = Memory::init_memory(args, card_index, action_index);
        Self {
            instructions,
            card_types,
            memory,
            board,
            log: vec![],
        }
    }

    pub fn execute(&mut self, instruction_limit: usize) -> Result<SingleExecutionResult, Error> {
        for _ in 0..instruction_limit {
            match self.run_one_instruction() {
                Ok(()) => {}
                Err(err) => match err {
                    InternalError::Halt => {
                        return Ok(SingleExecutionResult::Finished);
                    }
                    err => {
                        // Should be changed with Error trait
                        let error = Error {
                            instruction: self.memory.pc() as u32,
                            error: err,
                        };
                        return Err(error);
                    }
                },
            }
        }
        Ok(SingleExecutionResult::Unfinished)
    }

    pub fn resume_execution(
        instructions: InstructionRom<'a>,
        card_types: CardTypesRom<'a>,
        board: &'a mut Board,
        sealed_memory: Sealed<Memory>,
    ) -> Self {
        let memory = Sealed::<Memory>::release_data(sealed_memory);
        Self {
            instructions,
            card_types,
            memory,
            board,
            log: vec![],
        }
    }

    #[must_use]
    pub fn stop_execution(self) -> ExecutionResult {
        if self.is_halted() {
            ExecutionResult::Finished(self.log)
        } else {
            ExecutionResult::Unfinished(self.log, Sealed::<Memory> { data: self.memory })
        }
    }

    fn run_one_instruction(&mut self) -> Result<(), InternalError> {
        //TODO: better handing for Halt instruction.
        //Probably, we need to propogate InternalErrors from the instructions to this function.
        let instruction = self.instructions.fetch_instruction(self.memory.pc());
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
            VMCommand::LoadRegionAttr {
                region_index,
                attr_index,
            } => {
                // TODO: Check access violation
                todo!();
                let attr = self.board.regions[region_index as usize].attrs[attr_index as usize];
                self.memory.push_external(attr)
            }
            VMCommand::StoreRegionAttr {
                region_index,
                attr_index,
            } => {
                // TODO: Check access violation
                todo!();
                let value = self.memory.pop_external()?;
                self.log.push(Event::RegionChange {
                    region_index,
                    attr_index: attr_index,
                    previous_value: self.board.attrs[attr_index as usize],
                    new_value: value,
                });
                self.board.regions[region_index as usize].attrs[attr_index as usize] = value;
                Ok(())
            }
            VMCommand::LoadLocal { index } => self.memory.push_local(index as usize),
            VMCommand::StoreLocal { index } => self.memory.pop_local(index as usize),
            VMCommand::LoadArgument { index } => self.memory.push_argument(index as usize),
            VMCommand::StoreArgument { index } => self.memory.pop_argument(index as usize),
            VMCommand::Goto(instruction) => self.memory.jmp(instruction as usize),
            VMCommand::IfGoto(instruction) => self.memory.ifjmp(instruction as usize),
            VMCommand::Call { address, n_args } => {
                self.memory.call(address as usize, n_args as usize)
            }
            VMCommand::Function { n_locals } => self.memory.function(n_locals as usize),
            VMCommand::Return => self.memory.fn_return(),
            VMCommand::ReturnVoid => self.memory.return_void(),
            VMCommand::PushCardCount { region_index } => {
                // TODO: Check access violation
                todo!();
                let len = self.board.regions[region_index as usize].cards.len();
                self.memory.push_external(Word::Numeric(len as i32))
            }
            VMCommand::PushTypeCount => {
                let len = self.card_types.card_type_count();
                self.memory.push_external(Word::Numeric(len as i32))
            }
            VMCommand::PushCardCountWithCardType { region_index } => {
                self.push_card_count_with_type(region_index)
            }
            VMCommand::PushCardTypeByCardIndex { region_index } => {
                self.push_card_type_by_index(region_index)
            }
            VMCommand::PushCardTypeByCardId => unimplemented!(),
            VMCommand::LoadCardTypeAttrByTypeIndex { attr_index } => {
                self.push_card_type_attr_by_type_index(attr_index)
            }
            VMCommand::LoadCardTypeAttrByCardIndex {
                region_index,
                attr_index,
            } => self.push_card_type_attr_by_card_index(region_index, attr_index),
            VMCommand::LoadCardTypeAttrByCardId { attr_index } => unimplemented!(),
            VMCommand::LoadCardAttrByCardIndex {
                region_index,
                attr_index,
            } => self.push_card_attr_by_index(region_index, attr_index),
            VMCommand::LoadCardAttrByCardId { attr_index } => unimplemented!(),
            VMCommand::StoreCardAttrByCardIndex {
                region_index,
                attr_index,
            } => self.pop_card_attr(region_index, attr_index),
            VMCommand::StoreCardAttrByCardId { attr_index } => unimplemented!(),
            VMCommand::InstanceCardByTypeIndex { region_index } => {
                self.instantiate_card_by_type_index(region_index)
            }
            VMCommand::InstanceCardByTypeId { region_index } => {
                self.instantiate_card_by_type_id(region_index)
            }
            VMCommand::CallCardAction => self.call_card_action(),
            VMCommand::RemoveCardByIndex { region_index } => {
                self.remove_card_by_index(region_index)
            }
            VMCommand::RemoveCardById => self.remove_card_by_id(),
            VMCommand::Halt => Err(InternalError::Halt),
            VMCommand::ReadPlayerInput => unimplemented!(),
            VMCommand::ReadRandomInput => unimplemented!(),
        }
    }

    fn push_card_type_by_index(&mut self, region_index: u32) -> Result<(), InternalError> {
        // TODO: Check access violation
        todo!();
        let index = self.memory.pop_external_no_pc_inc()?;
        match index {
            Word::Numeric(i) => {
                let card_type =
                    self.board.regions[region_index as usize].cards[i as usize].card_type();
                let word = Word::Numeric(card_type as i32);
                self.memory.push_external(word)
            }
            Word::Boolean(_) => Err(InternalError::TypeMismatch),
        }
    }

    fn push_card_count_with_type(&mut self, region_index: u32) -> Result<(), InternalError> {
        todo!();
        let card_type = self.memory.pop_external_no_pc_inc()?;
        match card_type {
            Word::Numeric(id) => {
                // Word::Numeric contains i32, but card_type is u32, so convert is needed
                let count = self
                    .board
                    .cards
                    .iter()
                    .filter(|card| card.card_type() == id as u32)
                    .count();

                let word = Word::Numeric(count as i32);
                self.memory.push_external(word)
            }
            Word::Boolean(_) => Err(InternalError::TypeMismatch),
        }
    }

    fn push_card_type_attr_by_type_index(&mut self, attr_index: u32) -> Result<(), InternalError> {
        let type_index = self.memory.pop_external_no_pc_inc()?;
        match type_index {
            Word::Numeric(id) => {
                let card_type = self.card_types.card_type_by_type_index(id as usize);
                let attr_value = card_type.attr_by_index(attr_index as usize);

                let word = attr_value;
                self.memory.push_external(word)
            }
            Word::Boolean(_) => Err(InternalError::TypeMismatch),
        }
    }

    fn push_card_type_attr_by_card_index(
        &mut self,
        region_index: u16,
        attr_index: u16,
    ) -> Result<(), InternalError> {
        todo!();
        let card_index = self.memory.pop_external_no_pc_inc()?;
        match card_index {
            Word::Numeric(id) => {
                let card = &self.board.cards[id as usize];
                let card_type_id = card.card_type();
                let card_type = self
                    .card_types
                    .card_type_by_type_id(card_type_id)
                    .ok_or(InternalError::NoSuchType)?;
                let attr_value = card_type.attr_by_index(attr_index as usize);

                let word = attr_value;
                self.memory.push_external(word)
            }
            Word::Boolean(_) => Err(InternalError::TypeMismatch),
        }
    }

    fn push_card_attr_by_index(
        &mut self,
        region_index: u16,
        attr_index: u16,
    ) -> Result<(), InternalError> {
        todo!();
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

    fn pop_card_attr(&mut self, region_index: u16, attr_index: u16) -> Result<(), InternalError> {
        todo!();
        let card_index = self.memory.pop_external_no_pc_inc()?;
        match card_index {
            Word::Numeric(id) => {
                let card = &mut self.board.cards[id as usize];
                let attr_value = self.memory.pop_external()?;

                self.log.push(Event::CardChange {
                    region_index,
                    card_index: id as u32,
                    attr_index,
                    previous_value: card.attrs[attr_index as usize],
                    new_value: attr_value,
                });

                card.attrs[attr_index as usize] = attr_value;
                Ok(())
            }
            Word::Boolean(_) => Err(InternalError::TypeMismatch),
        }
    }

    fn instantiate_card_by_type_index(&mut self, region_index: u16) -> Result<(), InternalError> {
        todo!();
        let index = self.memory.pop_external()?;
        match index {
            Word::Numeric(index) => {
                let id = index.try_into().unwrap();
                let card = self
                    .card_types
                    .instance_card_by_type_index(id, self.board.generate_card_id())
                    .unwrap();
                self.board.cards.push(card);

                self.log.push(Event::AddCardByIndex {
                    card_index: (self.board.cards.len() - 1) as u32,
                    cargtype_index: id as u32,
                });
                Ok(())
            }
            Word::Boolean(_) => Err(InternalError::TypeMismatch),
        }
    }

    fn instantiate_card_by_type_id(&mut self, region_index: u16) -> Result<(), InternalError> {
        todo!();
        let index = self.memory.pop_external()?;
        match index {
            Word::Numeric(index) => {
                let id = index.try_into().unwrap();
                let card = self
                    .card_types
                    .instance_card_by_type_id(id, self.board.generate_card_id())
                    .unwrap();
                self.board.cards.push(card);

                self.log.push(Event::AddCardById {
                    card_index: (self.board.cards.len() - 1) as u32,
                    cargtype_id: id as u32,
                });
                Ok(())
            }
            Word::Boolean(_) => Err(InternalError::TypeMismatch),
        }
    }

    fn call_card_action(&mut self) -> Result<(), InternalError> {
        let action_index_word = self.memory.pop_external_no_pc_inc()?;
        let action_index =
            usize::try_from(action_index_word).map_err(|_| InternalError::TypeMismatch)?;

        let type_index_word = self.memory.pop_external_no_pc_inc()?;
        let type_index =
            usize::try_from(type_index_word).map_err(|_| InternalError::TypeMismatch)?;

        let card_type = self.card_types.card_type_by_type_index(type_index);
        let entry_point = card_type.action_entry_point(action_index);
        self.memory
            .call(entry_point.address(), entry_point.n_args())?;

        self.log.push(Event::CardActionStarted {
            cardtype_index: type_index as u32,
            action_index: action_index as u32,
            args: self.memory.args(),
        });
        Ok(())
    }

    fn remove_card_by_index(&mut self, region_index: u16) -> Result<(), InternalError> {
        todo!();
        let card_index_word = self.memory.pop_external()?;
        let card_index =
            usize::try_from(card_index_word).map_err(|_| InternalError::TypeMismatch)?;

        let card = self.board.cards.remove(card_index);

        self.log.push(Event::RemoveCard {
            card_id: card.id() as u32,
        });
        Ok(())
    }

    fn remove_card_by_id(&mut self) -> Result<(), InternalError> {
        todo!();
        let card_id = self.memory.pop_external()?;
        let card_id = u32::try_from(card_id).map_err(|_| InternalError::TypeMismatch)?;

        let board = &mut self.board;
        let log = &mut self.log;

        board.cards.retain(|card| {
            if card.id() == card_id {
                log.push(Event::RemoveCard {
                    card_id: card_id as u32,
                });
                true
            } else {
                false
            }
        });

        Ok(())
    }

    #[cfg(test)]
    fn release_memory(self) -> Memory {
        self.memory
    }

    #[must_use]
    pub fn is_halted(&self) -> bool {
        let instruction = self.instructions.fetch_instruction(self.memory.pc());
        instruction == VMCommand::Halt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::MemoryRegion;
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
        let common_region =
            MemoryRegion::with_data(0, vec![card1, card2, card3, card4], board_attrs);

        unsafe { Board::from_raw_parts(vec![common_region], 5) }
    }

    fn initial_board() -> Board {
        let card1 = type1().instantiate_card(1);
        let board_attrs = word_vec![0, 0, 0, false, false, false,];
        let common_region = MemoryRegion::with_data(0, vec![card1], board_attrs);
        unsafe { Board::from_raw_parts(vec![common_region], 1) }
    }

    #[test]
    fn init_empty_memory_vm() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::PushCardTypeByCardIndex { region_index: 0 },
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let card_types = vec![type1(), type2()];
        let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

        let mut board = testing_board();

        let args = vec![];
        let vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);
        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 0, 0, 0) };

        assert_eq!(memory, needed_memory);
    }

    #[test]
    #[ignore]
    fn push_type() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::PushCardTypeByCardIndex { region_index: 0 },
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let card_types = vec![type1(), type2()];
        let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

        let mut board = testing_board();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

        vm.execute(10).unwrap();
        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, 1], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let board_needed = testing_board();

        assert_eq!(board, board_needed);
    }

    #[test]
    #[ignore]
    fn push_card_count() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::PushCardCountWithCardType { region_index: 0 },
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let card_types = vec![type1(), type2()];
        let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

        let mut board = testing_board();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

        vm.execute(10).unwrap();

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
            VMCommand::LoadCardTypeAttrByTypeIndex { attr_index: 3 },
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let card_types = vec![type1(), type2()];
        let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

        let mut board = testing_board();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let board_needed = testing_board();

        assert_eq!(board, board_needed);
    }

    #[test]
    #[ignore]
    fn push_type_attr_by_card_index() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(1)),
            VMCommand::LoadCardTypeAttrByCardIndex {
                region_index: 0,
                attr_index: 3,
            },
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let card_types = vec![type1(), type2()];
        let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

        let mut board = testing_board();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let board_needed = testing_board();

        assert_eq!(board, board_needed);
    }

    #[test]
    #[ignore]
    fn push_attr() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(1)),
            VMCommand::LoadCardAttrByCardIndex {
                region_index: 0,
                attr_index: 3,
            },
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let card_types = vec![type1(), type2()];
        let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

        let mut board = testing_board();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let board_needed = testing_board();

        assert_eq!(board, board_needed);
    }

    #[test]
    #[ignore]
    fn pop_attr() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(42)),
            VMCommand::PushConstant(Word::Numeric(1)),
            VMCommand::StoreCardAttrByCardIndex {
                region_index: 0,
                attr_index: 3,
            },
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let card_types = vec![type1(), type2()];
        let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

        let mut board = testing_board();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 3, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut board_needed = testing_board();
        board_needed.cards[1].attrs[3] = Word::Numeric(42);

        assert_eq!(board, board_needed);
    }

    #[test]
    #[ignore]
    fn add_one_card_by_index() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(1)),
            VMCommand::InstanceCardByTypeIndex { region_index: 0 },
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let card_types = vec![type1(), type2()];
        let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

        let mut board = initial_board();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut board_needed = initial_board();
        let _ = board_needed.generate_card_id();
        let added_card = type2().instantiate_card(1);
        board_needed.regions[0].cards.push(added_card);

        assert_eq!(board, board_needed);
    }

    #[test]
    #[ignore]
    fn add_one_card_by_id() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::InstanceCardByTypeId { region_index: 0 },
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let card_types = vec![type1(), type2()];
        let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

        let mut board = initial_board();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut board_needed = initial_board();
        let _ = board_needed.generate_card_id();
        let added_card = type2().instantiate_card(1);
        board_needed.regions[0].cards.push(added_card);

        assert_eq!(board, board_needed);
    }

    #[test]
    #[ignore]
    fn add_one_card() {
        let instructions = vec![
            VMCommand::CallCardAction,
            VMCommand::Halt,
            //{
            VMCommand::Function { n_locals: 0 }, // Добавляет на доску одну карту типа 2
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::InstanceCardByTypeId { region_index: 0 },
            VMCommand::PushConstant(Word::Numeric(5)),
            VMCommand::Return,
            //}
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let card_types = vec![type1(), type2()];
        let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

        let mut board = initial_board();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![5], 0, 0, 1, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut board_needed = initial_board();
        let card = type2().instantiate_card(board_needed.generate_card_id());
        board_needed.regions[0].cards.push(card);

        assert_eq!(board, board_needed);
    }

    #[test]
    #[ignore]
    fn remove_one_card() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(0)),
            VMCommand::RemoveCardByIndex { region_index: 0 },
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let card_types = vec![type1(), type2()];
        let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

        let mut board = testing_board();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut board_needed = testing_board();
        board_needed.regions[0].cards.remove(0);

        assert_eq!(board, board_needed);
    }
}
