use crate::word::Word;
use std::convert::TryFrom;
use tinyvec::ArrayVec;

const STACK_SIZE: usize = 512;
type Stack = ArrayVec<[Word; STACK_SIZE]>;

#[derive(Debug)]
pub struct Memory {
    stack: Stack,
    lcl: usize,
    arg: usize,
    pc: usize,
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            stack: ArrayVec::<[Word; STACK_SIZE]>::new(),
            lcl: 0,
            arg: 0,
            pc: 0,
        }
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn jmp(&mut self, address: usize) {
        self.pc = address;
    }

    pub fn ifjmp(&mut self, address: usize) {
        let value = self.stack.pop();
        match value {
            Some(Word::Boolean(val)) => {
                if val {
                    self.pc = address;
                } else {
                    self.pc += 1;
                }
            }
            Some(Word::Numeric(_)) => {
                panic!("Type mismatch: attempted to use numerical value in boolean condition.");
            }
            None => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn add(&mut self) {
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
    pub fn sub(&mut self) {
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

    pub fn mul(&mut self) {
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
    pub fn div(&mut self) {
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
    pub fn rem(&mut self) {
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

    pub fn neg(&mut self) {
        let value = self.stack.pop();
        match value {
            Some(Word::Numeric(x)) => {
                self.stack.push(Word::Numeric(-x));
                self.pc += 1;
            }
            Some(Word::Boolean(_)) => {
                panic!("Attempted to negate boolean value.");
            }
            None => {
                panic!("Not enough values on the stack.")
            }
        }
    }

    pub fn push_external(&mut self, value: Word) {
        self.stack.push(value);
        self.pc += 1;
    }

    pub fn pop_external(&mut self) -> Word {
        let value = self.stack.pop().unwrap();
        self.pc += 1;
        value
    }

    pub fn push_local(&mut self, index: usize) {
        let value = self.stack[self.lcl + index];
        self.stack.push(value);
        self.pc += 1;
    }

    pub fn pop_local(&mut self, index: usize) {
        let value = self.stack.pop().unwrap();
        self.stack[self.lcl + index] = value;
        self.pc += 1;
    }

    pub fn push_argument(&mut self, index: usize) {
        let value = self.stack[self.arg + index];
        self.stack.push(value);
        self.pc += 1;
    }

    pub fn pop_argument(&mut self, index: usize) {
        let value = self.stack.pop().unwrap();
        self.stack[self.arg + index] = value;
        self.pc += 1;
    }

    pub fn eq(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Boolean(x == y));
                self.pc += 1;
            }
            (Some(Word::Boolean(_)), Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to check boolean values for equality. Use `XOR` instead.")
            }
            (Some(_), Some(_)) => {
                panic!("Type mismatch: attempted to compare boolean to numerical.")
            }
            (_, None) => {
                panic!("Not enough values on the stack.")
            }
            (None, _) => {
                unreachable!();
            }
        }
    }

    pub fn gt(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Boolean(x > y));
                self.pc += 1;
            }
            (Some(Word::Boolean(_)), Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to check boolean values for equality. Use `XOR` instead.")
            }
            (Some(_), Some(_)) => {
                panic!("Type mismatch: attempted to compare boolean to numerical.")
            }
            (_, None) => {
                panic!("Not enough values on the stack.")
            }
            (None, _) => {
                unreachable!();
            }
        }
    }

    pub fn lt(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Boolean(x < y));
                self.pc += 1;
            }
            (Some(Word::Boolean(_)), Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to check boolean values for equality. Use `XOR` instead.")
            }
            (Some(_), Some(_)) => {
                panic!("Type mismatch: attempted to compare boolean to numerical.")
            }
            (_, None) => {
                panic!("Not enough values on the stack.")
            }
            (None, _) => {
                unreachable!();
            }
        }
    }

    pub fn and(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Boolean(x)), Some(Word::Boolean(y))) => {
                self.stack.push(Word::Boolean(x && y));
                self.pc += 1;
            }
            (Some(Word::Numeric(_)), Some(Word::Numeric(_))) => {
                panic!("Type mismatch: attempted to AND numerical values.")
            }
            (Some(_), Some(_)) => {
                panic!("Type mismatch: attempted to AND boolean to numerical.")
            }
            (_, None) => {
                panic!("Not enough values on the stack.")
            }
            (None, _) => {
                unreachable!();
            }
        }
    }

    pub fn or(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Boolean(x)), Some(Word::Boolean(y))) => {
                self.stack.push(Word::Boolean(x || y));
                self.pc += 1;
            }
            (Some(Word::Numeric(_)), Some(Word::Numeric(_))) => {
                panic!("Type mismatch: attempted to OR numerical values.")
            }
            (Some(_), Some(_)) => {
                panic!("Type mismatch: attempted to OR boolean to numerical.")
            }
            (_, None) => {
                panic!("Not enough values on the stack.")
            }
            (None, _) => {
                unreachable!();
            }
        }
    }

    pub fn not(&mut self) {
        let value = self.stack.pop();
        match value {
            Some(Word::Boolean(x)) => {
                self.stack.push(Word::Boolean(!x));
                self.pc += 1;
            }
            Some(Word::Numeric(_)) => {
                panic!("Attempted to NOT numerical value.");
            }
            None => {
                panic!("Not enough values on the stack.")
            }
        }
    }

    pub fn call(&mut self, address: usize, n_args: usize) {
        let return_address = self.pc + 1;
        self.stack
            .push(Word::Numeric(i32::try_from(return_address).unwrap()));
        self.stack
            .push(Word::Numeric(i32::try_from(self.lcl).unwrap()));
        self.stack
            .push(Word::Numeric(i32::try_from(self.arg).unwrap()));
        self.arg = self.stack.len() - n_args - 3;
        self.lcl = self.stack.len();
        self.pc = address;
    }

    pub fn function(&mut self, n_locals: usize) {
        for _ in 0..n_locals {
            self.stack.push(Word::Numeric(0));
        }
        self.pc += 1;
    }

    pub fn fn_return(&mut self) {
        let frame = self.lcl;
        let return_address = self.stack[frame - 3].unwrap_numeric();
        let previous_lcl = self.stack[frame - 2].unwrap_numeric();
        let previous_arg = self.stack[frame - 1].unwrap_numeric();
        let return_value = self.stack.pop().unwrap();

        self.stack.truncate(self.arg);
        self.stack.push(return_value);
        self.lcl = previous_lcl as usize;
        self.arg = previous_arg as usize;
        self.pc = return_address as usize;
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

    mod data_flow {
        use super::*;

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

        #[test]
        fn push_local_data() {
            let mut mem = prepare_memory(
                vec![Word::Numeric(2), Word::Numeric(6), Word::Numeric(8)],
                1,
                0,
                0,
            );
            mem.push_local(0);
            mem.push_local(1);
            assert_eq!(
                mem.stack,
                array_vec!([Word; STACK_SIZE] =>
                    Word::Numeric(2),
                    Word::Numeric(6),
                    Word::Numeric(8),
                    Word::Numeric(6),
                    Word::Numeric(8))
            );
        }

        #[test]
        fn pop_local_data() {
            let mut mem = prepare_memory(
                vec![
                    Word::Numeric(2),
                    Word::Numeric(6),
                    Word::Numeric(8),
                    Word::Numeric(16),
                ],
                0,
                0,
                0,
            );
            mem.pop_local(0);
            mem.pop_local(1);
            assert_eq!(
                mem.stack,
                array_vec!([Word; STACK_SIZE] => Word::Numeric(16), Word::Numeric(8))
            );
        }

        #[test]
        fn push_argument_data() {
            let mut mem = prepare_memory(
                vec![Word::Numeric(2), Word::Numeric(6), Word::Numeric(8)],
                0,
                1,
                0,
            );
            mem.push_argument(0);
            mem.push_argument(1);
            assert_eq!(
                mem.stack,
                array_vec!([Word; STACK_SIZE] => Word::Numeric(2), Word::Numeric(6), Word::Numeric(8), Word::Numeric(6),Word::Numeric(8))
            );
        }

        #[test]
        fn pop_argument_data() {
            let mut mem = prepare_memory(
                vec![
                    Word::Numeric(2),
                    Word::Numeric(6),
                    Word::Numeric(8),
                    Word::Numeric(16),
                ],
                0,
                0,
                0,
            );
            mem.pop_argument(0);
            mem.pop_argument(1);
            assert_eq!(
                mem.stack,
                array_vec!([Word; STACK_SIZE] => Word::Numeric(16), Word::Numeric(8))
            );
        }
    }

    mod subroutine_calling {
        use super::*;

        #[test]
        fn call() {
            let mut mem = prepare_memory(vec![Word::Numeric(2), Word::Boolean(true)], 0, 1, 4);
            mem.call(16, 2);
            assert_eq!(
                mem.stack,
                array_vec!([Word; STACK_SIZE] => Word::Numeric(2), Word::Boolean(true), Word::Numeric(5), Word::Numeric(0), Word::Numeric(1))
            );
            assert_eq!(mem.lcl, 5);
            assert_eq!(mem.arg, 0);
            assert_eq!(mem.pc, 16);
        }

        #[test]
        fn fn_return() {
            let mut mem = prepare_memory(
                vec![
                    Word::Numeric(2),
                    Word::Boolean(true),
                    Word::Numeric(5),
                    Word::Numeric(0),
                    Word::Numeric(1),
                    Word::Boolean(false),
                ],
                5,
                0,
                16,
            );
            mem.fn_return();
            assert_eq!(
                mem.stack,
                array_vec!([Word; STACK_SIZE] => Word::Boolean(false))
            );
            assert_eq!(mem.lcl, 0);
            assert_eq!(mem.arg, 1);
            assert_eq!(mem.pc, 5);
        }

        #[test]
        fn function() {
            let mut mem = prepare_memory(vec![Word::Numeric(2), Word::Boolean(true)], 2, 0, 16);
            mem.function(3);
            assert_eq!(
                mem.stack,
                array_vec!([Word; STACK_SIZE] => Word::Numeric(2), Word::Boolean(true), Word::Numeric(0), Word::Numeric(0), Word::Numeric(0))
            );
            assert_eq!(mem.lcl, 2);
            assert_eq!(mem.arg, 0);
            assert_eq!(mem.pc, 17);
        }
    }
}
