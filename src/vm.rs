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
//! - Add
//! - Sub
//! - Div
//! - Mod
//! - Convert
//! - Neg
//! - Eq
//! - Gt
//! - Lt
//! - And
//! - Or
//! - Not
use tinyvec::ArrayVec;

/// Одна ячейка памяти на стеке может содержать либо число, либо логическое значение.
/// Операции будут проверять, что значение нужного типа, поэтому вызвать 1 + True нельзя, это
/// вызовет панику.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Word {
    Signed(i32),
    Boolean(bool),
    Unsigned(u32),
}

impl Default for Word {
    fn default() -> Self {
        Word::Signed(0)
    }
}

const STACK_SIZE: usize = 512;

#[derive(Debug)]
pub struct Memory {
    stack: ArrayVec<[Word; STACK_SIZE]>,
    lcl: usize,
    arg: usize,
}

impl Memory {
    fn new() -> Self {
        Memory {
            stack: ArrayVec::<[Word; STACK_SIZE]>::new(),
            lcl: 0,
            arg: 0,
        }
    }

    fn add(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Signed(x)), Some(Word::Signed(y))) => {
                self.stack.push(Word::Signed(x + y));
            }
            (Some(Word::Unsigned(x)), Some(Word::Unsigned(y))) => {
                self.stack.push(Word::Unsigned(x + y));
            }
            (_, None) => {
                panic!("Not enough values on the stack.")
            }
            (None, _) => {
                unreachable!();
            }
            (Some(Word::Unsigned(_)), Some(Word::Signed(_))) => {
                panic!("Type mismatch: attempted to add unsigned number to signed.\nUse `Convert` instruction to change Signed to Unsigned and vise versa.")
            }
            (Some(Word::Signed(_)), Some(Word::Unsigned(_))) => {
                panic!("Type mismatch: attempted to add signed number to unsigned.\nUse `Convert` instruction to change Signed to Unsigned and vise versa.")
            }
            (Some(Word::Boolean(_)), _) => {
                panic!("Type mismatch: attempted to add boolean values.")
            }
            (_, Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to add boolean values.")
            }
        }
    }

    /// Substracts the last value from the stack from the previous one
    fn sub(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Signed(x)), Some(Word::Signed(y))) => {
                self.stack.push(Word::Signed(x - y));
            }
            (Some(Word::Unsigned(x)), Some(Word::Unsigned(y))) => {
                self.stack.push(Word::Unsigned(x - y));
            }
            (_, None) => {
                panic!("Not enough values on the stack.")
            }
            (None, _) => {
                unreachable!();
                //panic!("Not enough values on the stack!")
            }
            (Some(Word::Unsigned(_)), Some(Word::Signed(_))) => {
                panic!("Type mismatch: attempted to substract unsigned number from signed.\nUse `Convert` instruction to change Signed to Unsigned and vise versa.")
            }
            (Some(Word::Signed(_)), Some(Word::Unsigned(_))) => {
                panic!("Type mismatch: attempted to substract signed number from unsigned.\nUse `Convert` instruction to change Signed to Unsigned and vise versa.")
            }
            (Some(Word::Boolean(_)), _) => {
                panic!("Type mismatch: attempted to substract boolean values.")
            }
            (_, Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to substract boolean values.")
            }
        }
    }

    fn mul(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Signed(x)), Some(Word::Signed(y))) => {
                self.stack.push(Word::Signed(x * y));
            }
            (Some(Word::Unsigned(x)), Some(Word::Unsigned(y))) => {
                self.stack.push(Word::Unsigned(x * y));
            }
            (_, None) => {
                panic!("Not enough values on the stack.")
            }
            (None, _) => {
                unreachable!();
            }
            (Some(Word::Unsigned(_)), Some(Word::Signed(_))) => {
                panic!("Type mismatch: attempted to multiply unsigned number by signed.\nUse `Convert` instruction to change Signed to Unsigned and vise versa.")
            }
            (Some(Word::Signed(_)), Some(Word::Unsigned(_))) => {
                panic!("Type mismatch: attempted to multiply signed number by unsigned.\nUse `Convert` instruction to change Signed to Unsigned and vise versa.")
            }
            (Some(Word::Boolean(_)), _) => {
                panic!("Type mismatch: attempted to multiply boolean values.")
            }
            (_, Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to multiply boolean values.")
            }
        }
    }
}

pub enum VMCommand {
    Add,
    Sub,
    Div,
    And,
    Or,
    Not,
    Halt,
}

const ROM_SIZE: usize = 512;

pub struct VM {
    rom: [VMCommand; ROM_SIZE],
    pc: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{self, MaybeUninit};
    use tinyvec::array_vec;
    fn prepare_memory(data: Vec<Word>, lcl: usize, arg: usize) -> Memory {
        assert!(lcl <= data.len());
        assert!(arg <= data.len());
        let mut stack = ArrayVec::<[Word; STACK_SIZE]>::new();
        stack.fill(data);
        Memory { stack, lcl, arg }
    }

    #[test]
    fn add_signed() {
        let mut mem = prepare_memory(vec![Word::Signed(1), Word::Signed(2)], 0, 0);
        mem.add();
        assert_eq!(mem.stack, array_vec!([Word; STACK_SIZE] => Word::Signed(3)));
    }

    #[test]
    fn add_unsigned() {
        let mut mem = prepare_memory(vec![Word::Unsigned(1), Word::Unsigned(2)], 0, 0);
        mem.add();
        assert_eq!(mem.stack, array_vec!([Word; STACK_SIZE] => Word::Unsigned(3)));
    }

    #[test]
    #[should_panic(expected = "Type mismatch: attempted to add unsigned number to signed.\nUse `Convert` instruction to change Signed to Unsigned and vise versa.")]
    fn add_unsigned_to_signed() {
        let mut mem = prepare_memory(vec![Word::Signed(1), Word::Unsigned(2)], 0, 0);
        mem.add();
    }
    
    #[test]
    #[should_panic(expected = "Type mismatch: attempted to add signed number to unsigned.\nUse `Convert` instruction to change Signed to Unsigned and vise versa.")]
    fn add_signed_to_unsigned() {
        let mut mem = prepare_memory(vec![Word::Unsigned(1), Word::Signed(2)], 0, 0);
        mem.add();
    }
    
    #[test]
    #[should_panic(expected = "Type mismatch: attempted to add boolean values.")]
    fn add_boolean() {
        let mut mem = prepare_memory(vec![Word::Signed(1), Word::Boolean(true)], 0, 0);
        mem.add();
    }

    #[test]
    #[should_panic(expected = "Not enough values on the stack.")]
    fn add_one_value_on_the_stack() {
        let mut mem = prepare_memory(vec![Word::Signed(1)], 0, 0);
        mem.add();
    }

    #[test]
    #[should_panic(expected = "Not enough values on the stack.")]
    fn add_empty_stack() {
        let mut mem = prepare_memory(Vec::<Word>::new(), 0, 0);
        mem.add();
    }

    #[test]
    fn sub_signed() {
        let mut mem = prepare_memory(vec![Word::Signed(1), Word::Signed(2)], 0, 0);
        mem.sub();
        assert_eq!(mem.stack, array_vec!([Word; STACK_SIZE] => Word::Signed(1)));
    }

    #[test]
    fn sub_unsigned() {
        let mut mem = prepare_memory(vec![Word::Unsigned(1), Word::Unsigned(2)], 0, 0);
        mem.sub();
        assert_eq!(mem.stack, array_vec!([Word; STACK_SIZE] => Word::Unsigned(1)));
    }

    #[test]
    #[should_panic(expected = "Type mismatch: attempted to substract unsigned number from signed.\nUse `Convert` instruction to change Signed to Unsigned and vise versa.")]
    fn sub_unsigned_to_unsigned() {
        let mut mem = prepare_memory(vec![Word::Signed(1), Word::Unsigned(2)], 0, 0);
        mem.sub();
    }
    
    #[test]
    #[should_panic(expected = "Type mismatch: attempted to substract boolean values.")]
    fn sub_boolean() {
        let mut mem = prepare_memory(vec![Word::Signed(1), Word::Boolean(true)], 0, 0);
        mem.sub();
    }

    #[test]
    #[should_panic(expected = "Not enough values on the stack.")]
    fn sub_one_value_on_the_stack() {
        let mut mem = prepare_memory(vec![Word::Signed(1)], 0, 0);
        mem.sub();
    }

    #[test]
    #[should_panic(expected = "Not enough values on the stack.")]
    fn sub_empty_stack() {
        let mut mem = prepare_memory(Vec::<Word>::new(), 0, 0);
        mem.sub();
    }

    #[test]
    fn mul_signed() {
        let mut mem = prepare_memory(vec![Word::Signed(4), Word::Signed(2)], 0, 0);
        mem.mul();
        assert_eq!(mem.stack, array_vec!([Word; STACK_SIZE] => Word::Signed(8)));
    }

    #[test]
    fn mul_unsigned() {
        let mut mem = prepare_memory(vec![Word::Unsigned(4), Word::Unsigned(2)], 0, 0);
        mem.mul();
        assert_eq!(mem.stack, array_vec!([Word; STACK_SIZE] => Word::Unsigned(8)));
    }

    #[test]
    #[should_panic(expected = "Type mismatch: attempted to multiply unsigned number by signed.\nUse `Convert` instruction to change Signed to Unsigned and vise versa.")]
    fn mul_unsigned_to_unsigned() {
        let mut mem = prepare_memory(vec![Word::Signed(1), Word::Unsigned(2)], 0, 0);
        mem.mul();
    }
    
    #[test]
    #[should_panic(expected = "Type mismatch: attempted to multiply boolean values.")]
    fn mul_boolean() {
        let mut mem = prepare_memory(vec![Word::Signed(1), Word::Boolean(true)], 0, 0);
        mem.mul();
    }

    #[test]
    #[should_panic(expected = "Not enough values on the stack.")]
    fn mul_one_value_on_the_stack() {
        let mut mem = prepare_memory(vec![Word::Signed(1)], 0, 0);
        mem.mul();
    }

    #[test]
    #[should_panic(expected = "Not enough values on the stack.")]
    fn mul_empty_stack() {
        let mut mem = prepare_memory(Vec::<Word>::new(), 0, 0);
        mem.mul();
    }
}
