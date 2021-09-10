//! # The Sorcery Virtual Machine
//! ## Memory model
//! Есть два основных региона памяти фиксированного размера: глобальный стек и память команд.
//! В памяти команд будет храниться байт-код, который SVM будет исполнять.
//! Глобальный стек будет использоваться как память общего назначения, в которой будут происходить
//! _промежуточные_ вычисления. Т.е. между ходами состояние стека сохраняться не должно.
//!
//! ### Memory segments
//! - local
//! - arguments
//! - constants
//! - player_attrs - позволяет выбрать атрибут игрока
//! - board_attrs - позволяет выбрать атрибут доски
//! - card_attrs - позволяет выбрать атрибут карты
//!
//! ## Instruction Set Architecture
//! - [x] Add
//! - [x] Sub
//! - [x] Div
//! - [x] Mul
//! - [ ] Rem
//! - [ ] Neg
//! - [ ] Eq
//! - [ ] Gt
//! - [ ] Lt
//! - [ ] And
//! - [ ] Or
//! - [ ] Not
use crate::board::Board;
use tinyvec::ArrayVec;

/// Одна ячейка памяти на стеке может содержать либо число, либо логическое значение.
/// Операции будут проверять, что значение нужного типа, поэтому вызвать 1 + True нельзя, это
/// вызовет панику.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Word {
    Numeric(i32),
    Boolean(bool),
}

impl Default for Word {
    fn default() -> Self {
        Word::Numeric(0)
    }
}

const STACK_SIZE: usize = 512;
type Stack = ArrayVec<[Word; STACK_SIZE]>;

#[derive(Debug)]
struct Memory {
    stack: Stack,
    lcl: usize,
    arg: usize,
    pc: usize,
}

impl Memory {
    fn new() -> Self {
        Memory {
            stack: ArrayVec::<[Word; STACK_SIZE]>::new(),
            lcl: 0,
            arg: 0,
            pc: 0,
        }
    }

    fn pc(&self) -> usize {
        self.pc
    }

    fn jmp(&mut self, address: usize) {
        self.pc = address;
    }

    fn add(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Numeric(x + y));
                self.pc += 1;
            }
            (Some(Word::Boolean(_)), _) => {
                panic!("Type mismatch: attempted to add boolean values.")
            }
            (_, Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to add boolean values.")
            }
            (_, None) => {
                panic!("Not enough values on the stack.")
            }
            (None, _) => {
                unreachable!();
            }
        }
    }

    /// Subtracts the last value from the stack from the previous one
    fn sub(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Numeric(x - y));
                self.pc += 1;
            }
            (Some(Word::Boolean(_)), _) => {
                panic!("Type mismatch: attempted to substract boolean values.")
            }
            (_, Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to substract boolean values.")
            }
            (_, None) => {
                panic!("Not enough values on the stack.")
            }
            (None, _) => {
                unreachable!();
            }
        }
    }

    fn mul(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Numeric(x * y));
                self.pc += 1;
            }
            (Some(Word::Boolean(_)), _) => {
                panic!("Type mismatch: attempted to multiply boolean values.")
            }
            (_, Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to multiply boolean values.")
            }
            (_, None) => {
                panic!("Not enough values on the stack.")
            }
            (None, _) => {
                unreachable!();
            }
        }
    }

    /// Divides the last value from the stack by the previous one, returns the quotient
    fn div(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Numeric(x / y));
                self.pc += 1;
            }
            (Some(Word::Boolean(_)), _) => {
                panic!("Type mismatch: attempted to divide boolean values.")
            }
            (_, Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to divide boolean values.")
            }
            (_, None) => {
                panic!("Not enough values on the stack.")
            }
            (None, _) => {
                unreachable!();
            }
        }
    }

    /// Divides the last value from the stack by the previous one, returnts the remainer
    fn rem(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Numeric(x % y));
                self.pc += 1;
            }
            (Some(Word::Boolean(_)), _) => {
                panic!("Type mismatch: attempted to take the remainer of the boolean values.")
            }
            (_, Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to take the remainer of the boolean values.")
            }
            (_, None) => {
                panic!("Not enough values on the stack.")
            }
            (None, _) => {
                unreachable!();
            }
        }
    }

    fn push_external(&mut self, value: Word) {
        self.stack.push(value);
        self.pc += 1;
    }

    fn pop_external(&mut self) -> Word {
        let value = self.stack.pop().unwrap();
        self.pc += 1;
        value
    }

    fn push_local(&mut self, index: usize) {
        let value = self.stack[self.lcl + index];
        self.stack.push(value);
        self.pc += 1;
    }

    fn pop_local(&mut self, index: usize) {
        let value = self.stack.pop().unwrap();
        self.stack[self.lcl + index] = value;
        self.pc += 1;
    }

    fn push_argument(&mut self, index: usize) {
        let value = self.stack[self.arg + index];
        self.stack.push(value);
        self.pc += 1;
    }

    fn pop_argument(&mut self, index: usize) {
        let value = self.stack.pop().unwrap();
        self.stack[self.arg + index] = value;
        self.pc += 1;
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VMCommand {
    Add,
    Sub,
    Div,
    Rem,
    Mul,
    //And,
    //Or,
    //Not,
    Halt,
    PushConstant(Word),
    PushBoardAttr { index: usize },
    PopBoardAttr { index: usize },
    PushLocal { index: usize },
    PopLocal { index: usize },
    PushArgument { index: usize },
    PopArgument { index: usize },
    Goto(usize),
    //IfGoto(usize),
}

impl Default for VMCommand {
    fn default() -> Self {
        VMCommand::Halt
    }
}

const ROM_SIZE: usize = 512;
type ROM = ArrayVec<[VMCommand; ROM_SIZE]>;

pub struct VM<'a> {
    rom: ROM,
    memory: Memory,
    board: &'a mut Board,
}

impl<'a> VM<'a> {
    pub fn new(rom: ROM, board: &'a mut Board) -> VM<'a> {
        VM {
            rom,
            memory: Memory::new(),
            board,
        }
    }
    fn run_one_instruction(&mut self) -> Result<(), ()> {
        //TODO: better handing for Halt instruction.
        //Probably, we need to propogate errors from the instructions to this function.
        let instruction = self.rom[self.memory.pc()];
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
            VMCommand::PushConstant(word) => {
                self.memory.push_external(word);
                Ok(())
            }
            VMCommand::PushBoardAttr { index } => {
                let attr = self.board.get_attr_by_index(index);
                self.memory.push_external(attr);
                Ok(())
            }
            VMCommand::PopBoardAttr { index } => {
                self.board.check_attr_index(index).unwrap();
                let value = self.memory.pop_external();
                self.board.set_attr_by_index(value, index);
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
            VMCommand::Halt => Err(()),
        }
    }

    pub fn execute(&mut self, instruction_limit: usize) {
        for _ in 0..instruction_limit {
            if self.run_one_instruction().is_err() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tinyvec::array_vec;
    fn prepare_memory(data: Vec<Word>, lcl: usize, arg: usize, pc: usize) -> Memory {
        assert!(lcl <= data.len());
        assert!(arg <= data.len());
        assert!(pc <= data.len());
        let mut stack = ArrayVec::<[Word; STACK_SIZE]>::new();
        stack.fill(data);
        Memory {
            stack,
            lcl,
            arg,
            pc,
        }
    }

    mod add {
        use super::*;
        use tinyvec::array_vec;

        #[test]
        fn numeric() {
            let mut mem = prepare_memory(vec![Word::Numeric(1), Word::Numeric(2)], 0, 0, 0);
            mem.add();
            assert_eq!(
                mem.stack,
                array_vec!([Word; STACK_SIZE] => Word::Numeric(3))
            );
        }

        #[test]
        #[should_panic(expected = "Type mismatch: attempted to add boolean values.")]
        fn boolean() {
            let mut mem = prepare_memory(vec![Word::Numeric(1), Word::Boolean(true)], 0, 0, 0);
            mem.add();
        }

        #[test]
        #[should_panic(expected = "Not enough values on the stack.")]
        fn one_value_on_the_stack() {
            let mut mem = prepare_memory(vec![Word::Numeric(1)], 0, 0, 0);
            mem.add();
        }

        #[test]
        #[should_panic(expected = "Not enough values on the stack.")]
        fn empty_stack() {
            let mut mem = prepare_memory(Vec::<Word>::new(), 0, 0, 0);
            mem.add();
        }
    }

    mod sub {
        use super::*;
        use tinyvec::array_vec;

        #[test]
        fn numeric() {
            let mut mem = prepare_memory(vec![Word::Numeric(1), Word::Numeric(2)], 0, 0, 0);
            mem.sub();
            assert_eq!(
                mem.stack,
                array_vec!([Word; STACK_SIZE] => Word::Numeric(1))
            );
        }

        #[test]
        #[should_panic(expected = "Type mismatch: attempted to substract boolean values.")]
        fn boolean() {
            let mut mem = prepare_memory(vec![Word::Numeric(1), Word::Boolean(true)], 0, 0, 0);
            mem.sub();
        }

        #[test]
        #[should_panic(expected = "Not enough values on the stack.")]
        fn one_value_on_the_stack() {
            let mut mem = prepare_memory(vec![Word::Numeric(1)], 0, 0, 0);
            mem.sub();
        }

        #[test]
        #[should_panic(expected = "Not enough values on the stack.")]
        fn empty_stack() {
            let mut mem = prepare_memory(Vec::<Word>::new(), 0, 0, 0);
            mem.sub();
        }
    }

    mod mul {
        use super::*;
        use tinyvec::array_vec;

        #[test]
        fn numeric() {
            let mut mem = prepare_memory(vec![Word::Numeric(4), Word::Numeric(2)], 0, 0, 0);
            mem.mul();
            assert_eq!(
                mem.stack,
                array_vec!([Word; STACK_SIZE] => Word::Numeric(8))
            );
        }

        #[test]
        #[should_panic(expected = "Type mismatch: attempted to multiply boolean values.")]
        fn boolean() {
            let mut mem = prepare_memory(vec![Word::Numeric(1), Word::Boolean(true)], 0, 0, 0);
            mem.mul();
        }

        #[test]
        #[should_panic(expected = "Not enough values on the stack.")]
        fn one_value_on_the_stack() {
            let mut mem = prepare_memory(vec![Word::Numeric(1)], 0, 0, 0);
            mem.mul();
        }

        #[test]
        #[should_panic(expected = "Not enough values on the stack.")]
        fn empty_stack() {
            let mut mem = prepare_memory(Vec::<Word>::new(), 0, 0, 0);
            mem.mul();
        }
    }

    mod div {
        use super::*;
        use tinyvec::array_vec;

        #[test]
        fn no_remainer() {
            let mut mem = prepare_memory(vec![Word::Numeric(2), Word::Numeric(6)], 0, 0, 0);
            mem.div();
            assert_eq!(
                mem.stack,
                array_vec!([Word; STACK_SIZE] => Word::Numeric(3))
            );
        }

        #[test]
        fn remainer() {
            let mut mem = prepare_memory(vec![Word::Numeric(2), Word::Numeric(7)], 0, 0, 0);
            mem.div();
            assert_eq!(
                mem.stack,
                array_vec!([Word; STACK_SIZE] => Word::Numeric(3))
            );
        }

        #[test]
        #[should_panic(expected = "Type mismatch: attempted to divide boolean values.")]
        fn boolean() {
            let mut mem = prepare_memory(vec![Word::Numeric(1), Word::Boolean(true)], 0, 0, 0);
            mem.div();
        }

        #[test]
        #[should_panic(expected = "Not enough values on the stack.")]
        fn one_value_on_the_stack() {
            let mut mem = prepare_memory(vec![Word::Numeric(1)], 0, 0, 0);
            mem.div();
        }

        #[test]
        #[should_panic(expected = "Not enough values on the stack.")]
        fn empty_stack() {
            let mut mem = prepare_memory(Vec::<Word>::new(), 0, 0, 0);
            mem.div();
        }
    }

    mod rem {
        use super::*;
        use tinyvec::array_vec;

        #[test]
        fn zero() {
            let mut mem = prepare_memory(vec![Word::Numeric(2), Word::Numeric(6)], 0, 0, 0);
            mem.rem();
            assert_eq!(
                mem.stack,
                array_vec!([Word; STACK_SIZE] => Word::Numeric(0))
            );
        }

        #[test]
        fn non_zero() {
            let mut mem = prepare_memory(vec![Word::Numeric(3), Word::Numeric(7)], 0, 0, 0);
            mem.rem();
            assert_eq!(
                mem.stack,
                array_vec!([Word; STACK_SIZE] => Word::Numeric(1))
            );
        }

        #[test]
        #[should_panic(
            expected = "Type mismatch: attempted to take the remainer of the boolean values."
        )]
        fn boolean() {
            let mut mem = prepare_memory(vec![Word::Numeric(1), Word::Boolean(true)], 0, 0, 0);
            mem.rem();
        }

        #[test]
        #[should_panic(expected = "Not enough values on the stack.")]
        fn one_value_on_the_stack() {
            let mut mem = prepare_memory(vec![Word::Numeric(1)], 0, 0, 0);
            mem.rem();
        }

        #[test]
        #[should_panic(expected = "Not enough values on the stack.")]
        fn empty_stack() {
            let mut mem = prepare_memory(Vec::<Word>::new(), 0, 0, 0);
            mem.rem();
        }
    }

    #[test]
    fn push_external_data() {
        let mut mem = Memory::new();
        mem.push_external(Word::Numeric(0));
        assert_eq!(
            mem.stack,
            array_vec!([Word; STACK_SIZE] => Word::Numeric(0))
        );
    }
    #[test]
    fn pop_external_data() {
        let mut mem = prepare_memory(vec![Word::Numeric(2), Word::Numeric(6)], 0, 0, 0);
        mem.pop_external();
        assert_eq!(
            mem.stack,
            array_vec!([Word; STACK_SIZE] => Word::Numeric(2))
        );
        mem.pop_external();
        assert_eq!(mem.stack, array_vec!([Word; STACK_SIZE]));
    }
}
