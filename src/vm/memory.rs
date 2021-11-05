use crate::word::Word;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct Memory {
    stack: Vec<Word>,
    lcl: usize,
    arg: usize,
    pc: usize,
}

impl<'a> Memory {
    pub fn init_memory(args: &'a [Word], card_index: u32, action_index: u32) -> Self {
        let mut stack = args.to_vec();

        stack.push(Word::Numeric(card_index as i32));
        stack.push(Word::Numeric(action_index as i32));

        Self {
            stack,
            lcl: 0,
            arg: 0,
            pc: 0,
        }
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn jmp(&mut self, address: usize) -> Result<(), VMError> {
        self.pc = address;
        Ok(())
    }

    pub fn ifjmp(&mut self, address: usize) -> Result<(), VMError> {
        let value = self.stack.pop();
        match value {
            Some(Word::Boolean(val)) => {
                if val {
                    self.pc = address;
                } else {
                    self.pc += 1;
                }
                Ok(())
            }
            Some(Word::Numeric(_)) => Err(VMError::TypeMismatch),
            None => Err(VMError::NotEnoughtValues),
        }
    }

    pub fn add(&mut self) -> Result<(), VMError> {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Numeric(x + y));
                self.pc += 1;
                Ok(())
            }
            (Some(Word::Boolean(_)), _) | (_, Some(Word::Boolean(_))) => Err(VMError::TypeMismatch),
            (_, None) | (None, _) => Err(VMError::NotEnoughtValues),
        }
    }

    /// Subtracts the last value from the stack from the previous one
    pub fn sub(&mut self) -> Result<(), VMError> {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Numeric(x - y));
                self.pc += 1;
                Ok(())
            }
            (Some(Word::Boolean(_)), _) | (_, Some(Word::Boolean(_))) => Err(VMError::TypeMismatch),
            (_, None) | (None, _) => Err(VMError::NotEnoughtValues),
        }
    }

    pub fn mul(&mut self) -> Result<(), VMError> {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Numeric(x * y));
                self.pc += 1;
                Ok(())
            }
            (Some(Word::Boolean(_)), _) | (_, Some(Word::Boolean(_))) => Err(VMError::TypeMismatch),
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    /// Divides the last value from the stack by the previous one, returns the quotient
    pub fn div(&mut self) -> Result<(), VMError> {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Numeric(x / y));
                self.pc += 1;
                Ok(())
            }
            (Some(Word::Boolean(_)), _) | (_, Some(Word::Boolean(_))) => Err(VMError::TypeMismatch),
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    /// Divides the last value from the stack by the previous one, returnts the remainer
    pub fn rem(&mut self) -> Result<(), VMError> {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Numeric(x % y));
                self.pc += 1;
                Ok(())
            }
            (Some(Word::Boolean(_)), _) | (_, Some(Word::Boolean(_))) => Err(VMError::TypeMismatch),
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn neg(&mut self) -> Result<(), VMError> {
        let value = self.stack.pop();
        match value {
            Some(Word::Numeric(x)) => {
                self.stack.push(Word::Numeric(-x));
                self.pc += 1;
                Ok(())
            }
            Some(Word::Boolean(_)) => Err(VMError::TypeMismatch),
            None => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn inc(&mut self) -> Result<(), VMError> {
        let value = self.stack.pop();
        match value {
            Some(Word::Numeric(x)) => {
                self.stack.push(Word::Numeric(x + 1));
                self.pc += 1;
                Ok(())
            }
            Some(Word::Boolean(_)) => Err(VMError::TypeMismatch),
            None => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn dec(&mut self) -> Result<(), VMError> {
        let value = self.stack.pop();
        match value {
            Some(Word::Numeric(x)) => {
                self.stack.push(Word::Numeric(x - 1));
                self.pc += 1;
                Ok(())
            }
            Some(Word::Boolean(_)) => Err(VMError::TypeMismatch),
            None => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn abs(&mut self) -> Result<(), VMError> {
        let value = self.stack.pop();
        match value {
            Some(Word::Numeric(x)) => {
                self.stack.push(Word::Numeric(x.abs()));
                self.pc += 1;
                Ok(())
            }
            Some(Word::Boolean(_)) => Err(VMError::TypeMismatch),
            None => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn push_external(&mut self, value: Word) -> Result<(), VMError> {
        self.stack.push(value);
        self.pc += 1;
        Ok(())
    }

    pub fn pop_external(&mut self) -> Result<Word, VMError> {
        match self.stack.pop() {
            Some(value) => {
                self.pc += 1;
                Ok(value)
            }
            None => Err(VMError::NotEnoughtValues),
        }
    }

    pub fn pop_external_no_pc_inc(&mut self) -> Result<Word, VMError> {
        self.stack.pop().ok_or(VMError::NotEnoughtValues)
    }

    pub fn push_local(&mut self, index: usize) -> Result<(), VMError> {
        let value = self.stack[self.lcl + index];
        self.stack.push(value);
        self.pc += 1;
        Ok(())
    }

    pub fn pop_local(&mut self, index: usize) -> Result<(), VMError> {
        match self.stack.pop() {
            Some(value) => {
                self.stack[self.lcl + index] = value;
                self.pc += 1;
                Ok(())
            }
            None => Err(VMError::NotEnoughtValues),
        }
    }

    pub fn push_argument(&mut self, index: usize) -> Result<(), VMError> {
        let value = self.stack[self.arg + index];
        self.stack.push(value);
        self.pc += 1;
        Ok(())
    }

    pub fn pop_argument(&mut self, index: usize) -> Result<(), VMError> {
        match self.stack.pop() {
            Some(value) => {
                self.stack[self.arg + index] = value;
                self.pc += 1;
                Ok(())
            }
            None => Err(VMError::NotEnoughtValues),
        }
    }

    pub fn equal(&mut self) -> Result<(), VMError> {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Boolean(x == y));
                self.pc += 1;
                Ok(())
            }
            (Some(_), Some(_)) => Err(VMError::TypeMismatch),
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn gt(&mut self) -> Result<(), VMError> {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Boolean(x > y));
                self.pc += 1;
                Ok(())
            }
            (Some(_), Some(_)) => Err(VMError::TypeMismatch),
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn lt(&mut self) -> Result<(), VMError> {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Boolean(x < y));
                self.pc += 1;
                Ok(())
            }
            (Some(_), Some(_)) => Err(VMError::TypeMismatch),
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn and(&mut self) -> Result<(), VMError> {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Boolean(x)), Some(Word::Boolean(y))) => {
                self.stack.push(Word::Boolean(x && y));
                self.pc += 1;
                Ok(())
            }
            (Some(_), Some(_)) => Err(VMError::TypeMismatch),
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn or(&mut self) -> Result<(), VMError> {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Boolean(x)), Some(Word::Boolean(y))) => {
                self.stack.push(Word::Boolean(x || y));
                self.pc += 1;
                Ok(())
            }
            (Some(_), Some(_)) => Err(VMError::TypeMismatch),
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn not(&mut self) -> Result<(), VMError> {
        let value = self.stack.pop();
        match value {
            Some(Word::Boolean(x)) => {
                self.stack.push(Word::Boolean(!x));
                self.pc += 1;
                Ok(())
            }
            Some(Word::Numeric(_)) => Err(VMError::TypeMismatch),
            None => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn call(&mut self, address: usize, n_args: usize) -> Result<(), VMError> {
        let return_address = self.pc + 1;
        self.stack.push(Word::Numeric(return_address as i32));
        self.stack.push(Word::Numeric(self.lcl as i32));
        self.stack.push(Word::Numeric(self.arg as i32));
        self.lcl = self.stack.len();
        self.arg = self.stack.len() - n_args - 3;
        self.pc = address;
        Ok(())
    }

    pub fn function(&mut self, n_locals: usize) -> Result<(), VMError> {
        for _ in 0..n_locals {
            self.stack.push(Word::Numeric(0));
        }
        self.pc += 1;
        Ok(())
    }

    pub fn fn_return(&mut self) -> Result<(), VMError> {
        let frame = self.lcl;
        let return_address =
            i32::try_from(self.stack[frame - 3]).map_err(|_| VMError::TypeMismatch)?;
        let previous_lcl =
            i32::try_from(self.stack[frame - 2]).map_err(|_| VMError::TypeMismatch)?;
        let previous_arg =
            i32::try_from(self.stack[frame - 1]).map_err(|_| VMError::TypeMismatch)?;
        let return_value = self.stack.pop().ok_or(VMError::NotEnoughtValues)?;

        self.stack.truncate(self.arg);
        self.stack.push(return_value);
        self.lcl = previous_lcl as usize;
        self.arg = previous_arg as usize;
        self.pc = return_address as usize;
        Ok(())
    }

    pub fn return_void(&mut self) -> Result<(), VMError> {
        let frame = self.lcl;
        let return_address =
            i32::try_from(self.stack[frame - 3]).map_err(|_| VMError::TypeMismatch)?;
        let previous_lcl =
            i32::try_from(self.stack[frame - 2]).map_err(|_| VMError::TypeMismatch)?;
        let previous_arg =
            i32::try_from(self.stack[frame - 1]).map_err(|_| VMError::TypeMismatch)?;

        self.stack.truncate(self.arg);
        self.lcl = previous_lcl as usize;
        self.arg = previous_arg as usize;
        self.pc = return_address as usize;
        Ok(())
    }

    #[cfg(test)]
    pub unsafe fn from_raw_parts(stack: Vec<Word>, lcl: usize, arg: usize, pc: usize) -> Self {
        assert!(lcl <= stack.len());
        assert!(arg <= stack.len());
        Self {
            stack,
            lcl,
            arg,
            pc,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum VMError {
    Halt,
    NotEnoughtValues,
    TypeMismatch,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::word_vec;

    mod arithmetic {
        use super::*;

        mod add {
            use super::*;

            #[test]
            fn numeric() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, 2], 0, 0, 0) };
                mem.add();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(3)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to add boolean values.")]
            fn boolean_first() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, true], 0, 0, 0) };
                mem.add();
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to add boolean values.")]
            fn boolean_second() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![true, 1], 0, 0, 0) };
                mem.add();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn one_value_on_the_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0) };
                mem.add();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.add();
            }
        }

        mod sub {
            use super::*;

            #[test]
            fn numeric() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, 2], 0, 0, 0) };
                mem.sub();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(1)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to substract boolean values.")]
            fn boolean() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, true], 0, 0, 0) };
                mem.sub();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn one_value_on_the_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0) };
                mem.sub();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.sub();
            }
        }

        mod mul {
            use super::*;

            #[test]
            fn numeric() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![4, 2], 0, 0, 0) };
                mem.mul();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(8)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to multiply boolean values.")]
            fn boolean() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, true], 0, 0, 0) };
                mem.mul();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn one_value_on_the_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0) };
                mem.mul();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.mul();
            }
        }

        mod div {
            use super::*;

            #[test]
            fn no_remainer() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6], 0, 0, 0) };
                mem.div();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(3)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            fn remainer() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 7], 0, 0, 0) };
                mem.div();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(3)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to divide boolean values.")]
            fn boolean() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, true], 0, 0, 0) };
                mem.div();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn one_value_on_the_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0) };
                mem.div();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.div();
            }
        }

        mod rem {
            use super::*;

            #[test]
            fn zero() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6], 0, 0, 0) };
                mem.rem();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(0)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            fn non_zero() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![3, 7], 0, 0, 0) };
                mem.rem();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(1)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(
                expected = "Type mismatch: attempted to take the remainer of the boolean values."
            )]
            fn boolean() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, true], 0, 0, 0) };
                mem.rem();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn one_value_on_the_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0) };
                mem.rem();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.rem();
            }
        }

        mod neg {
            use super::*;

            #[test]
            fn numeric() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, 2], 0, 0, 0) };
                mem.neg();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(1), Word::Numeric(-2)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(expected = "Attempted to negate boolean value.")]
            fn boolean() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, true], 0, 0, 0) };
                mem.neg();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.neg();
            }
        }

        mod inc {
            use super::*;

            #[test]
            fn numeric() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, 2], 0, 0, 0) };
                mem.inc();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(1), Word::Numeric(3)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(expected = "Attempted to increment boolean value.")]
            fn boolean() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, true], 0, 0, 0) };
                mem.inc();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.inc();
            }
        }

        mod dec {
            use super::*;

            #[test]
            fn numeric() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, 2], 0, 0, 0) };
                mem.dec();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(1), Word::Numeric(1)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(expected = "Attempted to decrement boolean value.")]
            fn boolean() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, true], 0, 0, 0) };
                mem.dec();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.dec();
            }
        }

        mod abs {
            use super::*;

            #[test]
            fn positive() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, 2], 0, 0, 0) };
                mem.abs();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(1), Word::Numeric(2)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            fn negative() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, -2], 0, 0, 0) };
                mem.abs();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(1), Word::Numeric(2)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(expected = "Attempted to find modulus of boolean value.")]
            fn boolean() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, true], 0, 0, 0) };
                mem.abs();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.abs();
            }
        }
    }

    mod logic {
        use super::*;

        mod eq {
            use super::*;

            #[test]
            fn equal() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![4, 4], 0, 0, 0) };
                mem.equal();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(true)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            fn non_equal() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![5, 4], 0, 0, 0) };
                mem.equal();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(false)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to compare boolean to numerical.")]
            fn type_mismatch() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, true], 0, 0, 0) };
                mem.equal();
            }

            #[test]
            #[should_panic(
                expected = "Type mismatch: attempted to check boolean values for equality."
            )]
            fn boolean() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![true, true], 0, 0, 0) };
                mem.equal();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn one_value_on_the_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0) };
                mem.equal();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.equal();
            }
        }

        mod gt {
            use super::*;

            #[test]
            fn equal() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![4, 4], 0, 0, 0) };
                mem.gt();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(false)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            fn smaller() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![5, 4], 0, 0, 0) };
                mem.gt();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(false)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            fn bigger() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![4, 5], 0, 0, 0) };
                mem.gt();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(true)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to compare boolean to numerical.")]
            fn type_mismatch() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, true], 0, 0, 0) };
                mem.gt();
            }

            #[test]
            #[should_panic(
                expected = "Type mismatch: attempted to check boolean values for equality."
            )]
            fn boolean() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![true, true], 0, 0, 0) };
                mem.gt();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn one_value_on_the_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0) };
                mem.gt();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.gt();
            }
        }

        mod lt {
            use super::*;

            #[test]
            fn equal() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![4, 4], 0, 0, 0) };
                mem.lt();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(false)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            fn smaller() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![5, 4], 0, 0, 0) };
                mem.lt();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(true)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            fn bigger() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![4, 5], 0, 0, 0) };
                mem.lt();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(false)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to compare boolean to numerical.")]
            fn type_mismatch() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, true], 0, 0, 0) };
                mem.lt();
            }

            #[test]
            #[should_panic(
                expected = "Type mismatch: attempted to check boolean values for equality."
            )]
            fn boolean() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![true, true], 0, 0, 0) };
                mem.lt();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn one_value_on_the_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0) };
                mem.lt();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.lt();
            }
        }

        mod and {
            use super::*;

            #[test]
            fn false_false() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![false, false], 0, 0, 0) };
                mem.and();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(false)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            fn false_true() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![false, true], 0, 0, 0) };
                mem.and();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(false)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            fn true_true() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![true, true], 0, 0, 0) };
                mem.and();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(true)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to AND boolean to numerical.")]
            fn type_mismatch() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, true], 0, 0, 0) };
                mem.and();
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to AND numerical values.")]
            fn boolean() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, 2], 0, 0, 0) };
                mem.and();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn one_value_on_the_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0) };
                mem.and();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.and();
            }
        }

        mod or {
            use super::*;

            #[test]
            fn false_false() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![false, false], 0, 0, 0) };
                mem.or();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(false)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            fn false_true() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![false, true], 0, 0, 0) };
                mem.or();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(true)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            fn true_true() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![true, true], 0, 0, 0) };
                mem.or();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(true)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to OR boolean to numerical.")]
            fn type_mismatch() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, true], 0, 0, 0) };
                mem.or();
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to OR numerical values.")]
            fn boolean() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1, 2], 0, 0, 0) };
                mem.or();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn one_value_on_the_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0) };
                mem.or();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.or();
            }
        }

        mod not {
            use super::*;

            #[test]
            fn test_false() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![false], 0, 0, 0) };
                mem.not();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(true)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            fn test_true() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![true], 0, 0, 0) };
                mem.not();
                pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(false)));
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(expected = "Attempted to NOT numerical value.")]
            fn numerical() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0) };
                mem.not();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.not();
            }
        }
    }

    mod data_flow {
        use super::*;

        #[test]
        fn push_external_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 0) };
            mem.push_external(Word::Numeric(0));
            pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(0)));
            pretty_assertions::assert_eq!(mem.pc, 1);
        }

        #[test]
        fn pop_external_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6], 0, 0, 0) };
            mem.pop_external();
            pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(2)));
            pretty_assertions::assert_eq!(mem.pc, 1);
            mem.pop_external();
            pretty_assertions::assert_eq!(mem.stack, vec![]);
            pretty_assertions::assert_eq!(mem.pc, 2);
        }

        #[test]
        fn pop_external_no_inc() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6], 0, 0, 0) };
            mem.pop_external_no_pc_inc();
            pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(2)));
            pretty_assertions::assert_eq!(mem.pc, 0);
            mem.pop_external_no_pc_inc();
            pretty_assertions::assert_eq!(mem.stack, vec![]);
            pretty_assertions::assert_eq!(mem.pc, 0);
        }

        #[test]
        fn push_local_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8], 1, 0, 0) };
            mem.push_local(0);
            mem.push_local(1);
            pretty_assertions::assert_eq!(
                mem.stack,
                vec!(
                    Word::Numeric(2),
                    Word::Numeric(6),
                    Word::Numeric(8),
                    Word::Numeric(6),
                    Word::Numeric(8)
                )
            );
            pretty_assertions::assert_eq!(mem.pc, 2);
        }

        #[test]
        fn pop_local_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8, 16,], 0, 0, 0) };
            mem.pop_local(0);
            mem.pop_local(1);
            pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(16), Word::Numeric(8)));
            pretty_assertions::assert_eq!(mem.pc, 2);
        }

        #[test]
        fn push_argument_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8], 0, 1, 0) };
            mem.push_argument(0);
            mem.push_argument(1);
            pretty_assertions::assert_eq!(
                mem.stack,
                vec!(
                    Word::Numeric(2),
                    Word::Numeric(6),
                    Word::Numeric(8),
                    Word::Numeric(6),
                    Word::Numeric(8)
                )
            );
            pretty_assertions::assert_eq!(mem.pc, 2);
        }

        #[test]
        fn pop_argument_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8, 16,], 0, 0, 0) };
            mem.pop_argument(0);
            mem.pop_argument(1);
            pretty_assertions::assert_eq!(mem.stack, vec!(Word::Numeric(16), Word::Numeric(8)));
            pretty_assertions::assert_eq!(mem.pc, 2);
        }
    }

    mod control_flow {
        use super::*;

        #[test]
        fn call() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, true], 0, 1, 4) };
            mem.call(16, 2);
            let mem_expected =
                unsafe { Memory::from_raw_parts(word_vec![2, true, 5, 0, 1], 5, 0, 16) };
            pretty_assertions::assert_eq!(mem, mem_expected);
        }
        #[test]
        fn call_no_args() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 4) };
            mem.call(16, 0);
            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![5, 0, 0], 3, 0, 16) };
            pretty_assertions::assert_eq!(mem, mem_expected);
        }

        #[test]
        fn fn_return() {
            let mut mem =
                unsafe { Memory::from_raw_parts(word_vec![2, true, 5, 0, 1, false], 5, 0, 16) };
            mem.fn_return();
            pretty_assertions::assert_eq!(mem.stack, vec!(Word::Boolean(false)));
            pretty_assertions::assert_eq!(mem.lcl, 0);
            pretty_assertions::assert_eq!(mem.arg, 1);
            pretty_assertions::assert_eq!(mem.pc, 5);
        }

        #[test]
        fn function() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, true], 2, 0, 16) };
            mem.function(3);
            pretty_assertions::assert_eq!(
                mem.stack,
                vec!(
                    Word::Numeric(2),
                    Word::Boolean(true),
                    Word::Numeric(0),
                    Word::Numeric(0),
                    Word::Numeric(0)
                )
            );
            pretty_assertions::assert_eq!(mem.lcl, 2);
            pretty_assertions::assert_eq!(mem.arg, 0);
            pretty_assertions::assert_eq!(mem.pc, 17);
        }

        mod ifjmp {
            use super::*;
            #[test]
            fn conditional_jump_successful() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![true], 0, 0, 0) };
                mem.ifjmp(10);
                pretty_assertions::assert_eq!(mem.stack, Vec::<Word>::new());
                pretty_assertions::assert_eq!(mem.pc, 10);
            }

            #[test]
            fn conditional_jump_unsuccessful() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![false], 0, 0, 0) };
                mem.ifjmp(10);
                pretty_assertions::assert_eq!(mem.stack, Vec::<Word>::new());
                pretty_assertions::assert_eq!(mem.pc, 1);
            }

            #[test]
            #[should_panic(
                expected = "Type mismatch: attempted to use numerical value in boolean condition."
            )]
            fn type_mismatch() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0) };
                mem.ifjmp(10);
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                mem.ifjmp(10);
            }
        }
    }
}
