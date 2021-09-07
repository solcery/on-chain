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
//! - [ ] Div
//! - [x] Mul
//! - [ ] Mod
//! - [ ] Neg
//! - [ ] Eq
//! - [ ] Gt
//! - [ ] Lt
//! - [ ] And
//! - [ ] Or
//! - [ ] Not
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
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Numeric(x + y));
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

    /// Substracts the last value from the stack from the previous one
    fn sub(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Numeric(x - y));
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
    fn add_numeric() {
        let mut mem = prepare_memory(vec![Word::Numeric(1), Word::Numeric(2)], 0, 0);
        mem.add();
        assert_eq!(mem.stack, array_vec!([Word; STACK_SIZE] => Word::Numeric(3)));
    }

    #[test]
    #[should_panic(expected = "Type mismatch: attempted to add boolean values.")]
    fn add_boolean() {
        let mut mem = prepare_memory(vec![Word::Numeric(1), Word::Boolean(true)], 0, 0);
        mem.add();
    }

    #[test]
    #[should_panic(expected = "Not enough values on the stack.")]
    fn add_one_value_on_the_stack() {
        let mut mem = prepare_memory(vec![Word::Numeric(1)], 0, 0);
        mem.add();
    }

    #[test]
    #[should_panic(expected = "Not enough values on the stack.")]
    fn add_empty_stack() {
        let mut mem = prepare_memory(Vec::<Word>::new(), 0, 0);
        mem.add();
    }

    #[test]
    fn sub_numeric() {
        let mut mem = prepare_memory(vec![Word::Numeric(1), Word::Numeric(2)], 0, 0);
        mem.sub();
        assert_eq!(mem.stack, array_vec!([Word; STACK_SIZE] => Word::Numeric(1)));
    }

    #[test]
    #[should_panic(expected = "Type mismatch: attempted to substract boolean values.")]
    fn sub_boolean() {
        let mut mem = prepare_memory(vec![Word::Numeric(1), Word::Boolean(true)], 0, 0);
        mem.sub();
    }

    #[test]
    #[should_panic(expected = "Not enough values on the stack.")]
    fn sub_one_value_on_the_stack() {
        let mut mem = prepare_memory(vec![Word::Numeric(1)], 0, 0);
        mem.sub();
    }

    #[test]
    #[should_panic(expected = "Not enough values on the stack.")]
    fn sub_empty_stack() {
        let mut mem = prepare_memory(Vec::<Word>::new(), 0, 0);
        mem.sub();
    }

    #[test]
    fn mul_numeric() {
        let mut mem = prepare_memory(vec![Word::Numeric(4), Word::Numeric(2)], 0, 0);
        mem.mul();
        assert_eq!(mem.stack, array_vec!([Word; STACK_SIZE] => Word::Numeric(8)));
    }
    
    #[test]
    #[should_panic(expected = "Type mismatch: attempted to multiply boolean values.")]
    fn mul_boolean() {
        let mut mem = prepare_memory(vec![Word::Numeric(1), Word::Boolean(true)], 0, 0);
        mem.mul();
    }

    #[test]
    #[should_panic(expected = "Not enough values on the stack.")]
    fn mul_one_value_on_the_stack() {
        let mut mem = prepare_memory(vec![Word::Numeric(1)], 0, 0);
        mem.mul();
    }

    #[test]
    #[should_panic(expected = "Not enough values on the stack.")]
    fn mul_empty_stack() {
        let mut mem = prepare_memory(Vec::<Word>::new(), 0, 0);
        mem.mul();
    }
}
