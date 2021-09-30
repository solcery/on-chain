//! # The Sorcery Virtual Machine

use crate::board::Board;
use crate::rom::Rom;
use crate::word::Word;
use std::convert::TryInto;

mod memory;
use memory::Memory;

#[derive(Debug, Eq, PartialEq)]
pub struct Sealed<T> {
    data: T,
}

impl<T> Sealed<T> {
    fn release_data(self) -> T {
        self.data
    }
}

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
    /// Card index and action index should be placed on the stack
    CallCardAction,
    RemoveCardByIndex,
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
    pub fn init_vm(
        rom: &'a Rom,
        board: &'a mut Board,
        args: Vec<Word>,
        card_index: i32,
        action_index: i32,
    ) -> VM<'a> {
        let memory = Memory::init_memory(args, card_index, action_index);
        VM { rom, memory, board }
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
    ) -> VM<'a> {
        let memory = Sealed::<Memory>::release_data(sealed_memory);
        VM { rom, memory, board }
    }

    pub fn stop_execution(self) -> Sealed<Memory> {
        Sealed::<Memory> { data: self.memory }
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
                self.memory.equal();
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
            VMCommand::CallCardAction => {
                self.call_card_action();
                Ok(())
            }
            VMCommand::RemoveCardByIndex => {
                self.remove_card_by_index();
                Ok(())
            }
            VMCommand::Halt => Err(()),
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
                let card = self
                    .rom
                    .instance_card_by_type_index(id, self.board.generate_card_id())
                    .unwrap();
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
                let card = self
                    .rom
                    .instance_card_by_type_id(id, self.board.generate_card_id())
                    .unwrap();
                self.board.cards.push(card);
            }
            Word::Boolean(_) => {
                panic!("Type mismath: bool can not be interpreted as index.");
            }
        }
    }

    fn call_card_action(&mut self) {
        let action_index: usize = self
            .memory
            .pop_external_no_pc_inc()
            .unwrap_numeric()
            .try_into()
            .unwrap();
        let card_index: usize = self
            .memory
            .pop_external_no_pc_inc()
            .unwrap_numeric()
            .try_into()
            .unwrap();
        let card = &self.board.cards[card_index];
        let type_id = card.card_type();
        let card_type = self.rom.card_type_by_type_id(type_id).unwrap();
        let entry_point = card_type.action_entry_point(action_index);
        self.memory
            .call(entry_point.address(), entry_point.n_args());
    }

    fn remove_card_by_index(&mut self) {
        let card_index: usize = self
            .memory
            .pop_external()
            .unwrap_numeric()
            .try_into()
            .unwrap();
        self.board.cards.remove(card_index);
    }

    #[cfg(test)]
    fn release_memory(self) -> Memory {
        self.memory
    }

    #[cfg(test)]
    fn is_halted(&self) -> bool {
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

        let vm = VM::init_vm(&rom, &mut board, vec![], 0, 0);
        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 0) };

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

        let mut vm = VM::init_vm(&rom, &mut board, vec![], 0, 0);

        vm.execute(10);
        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, 1], 0, 0, 2) };

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

        let mut vm = VM::init_vm(&rom, &mut board, vec![], 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, 2], 0, 0, 2) };

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

        let mut vm = VM::init_vm(&rom, &mut board, vec![], 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2) };

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

        let mut vm = VM::init_vm(&rom, &mut board, vec![], 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2) };

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

        let mut vm = VM::init_vm(&rom, &mut board, vec![], 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2) };

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

        let mut vm = VM::init_vm(&rom, &mut board, vec![], 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 3) };

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

        let mut vm = VM::init_vm(&rom, &mut board, vec![], 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2) };

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

        let mut vm = VM::init_vm(&rom, &mut board, vec![], 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2) };

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

        let mut vm = VM::init_vm(&rom, &mut board, vec![], 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![5], 0, 0, 1) };

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

        let mut vm = VM::init_vm(&rom, &mut board, vec![], 0, 0);

        vm.execute(10);

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2) };

        assert_eq!(memory, needed_memory);

        let mut board_needed = testing_board();
        board_needed.cards.remove(0);

        assert_eq!(board, board_needed);
    }
}
