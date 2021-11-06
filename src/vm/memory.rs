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
            (_, None) | (None, _) => Err(VMError::NotEnoughtValues),
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
            (_, None) | (None, _) => Err(VMError::NotEnoughtValues),
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
            (_, None) | (None, _) => Err(VMError::NotEnoughtValues),
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
            None => Err(VMError::NotEnoughtValues),
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
            None => Err(VMError::NotEnoughtValues),
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
            None => Err(VMError::NotEnoughtValues),
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
            None => Err(VMError::NotEnoughtValues),
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
            (_, None) | (None, _) => Err(VMError::NotEnoughtValues),
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
            (_, None) | (None, _) => Err(VMError::NotEnoughtValues),
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
            (_, None) | (None, _) => Err(VMError::NotEnoughtValues),
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
            (_, None) | (None, _) => Err(VMError::NotEnoughtValues),
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
            (_, None) | (None, _) => Err(VMError::NotEnoughtValues),
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
            None => Err(VMError::NotEnoughtValues),
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

    mod correct_operation {
        use super::*;
        use pretty_assertions::assert_eq;

        macro_rules! correct_operation {
            ($method:ident, $stack:expr, $expected_stack: expr) => {
                #[test]
                fn $method() {
                    let mut mem = unsafe { Memory::from_raw_parts($stack, 0, 0, 0) };
                    mem.$method().unwrap();
                    let mem_expected = unsafe { Memory::from_raw_parts($expected_stack, 0, 0, 1) };
                    assert_eq!(mem, mem_expected);
                }
            };
            ($method:ident, $stack:expr, $expected_stack: expr, $name: ident) => {
                #[test]
                fn $name() {
                    let mut mem = unsafe { Memory::from_raw_parts($stack, 0, 0, 0) };
                    mem.$method().unwrap();
                    let mem_expected = unsafe { Memory::from_raw_parts($expected_stack, 0, 0, 1) };
                    assert_eq!(mem, mem_expected);
                }
            };
        }

        // Arithmetic
        correct_operation!(add, word_vec![1, 2], word_vec![3]);
        correct_operation!(sub, word_vec![1, 2], word_vec![1]);
        correct_operation!(mul, word_vec![4, 2], word_vec![8]);
        correct_operation!(div, word_vec![2, 6], word_vec![3], div_no_remainer);
        correct_operation!(div, word_vec![2, 7], word_vec![3], div_remainer);
        correct_operation!(rem, word_vec![2, 6], word_vec![0], rem_zero);
        correct_operation!(rem, word_vec![3, 7], word_vec![1], rem_nonzero);
        correct_operation!(neg, word_vec![1, 2], word_vec![1, -2]);
        correct_operation!(inc, word_vec![1], word_vec![2]);
        correct_operation!(dec, word_vec![1], word_vec![0]);
        correct_operation!(abs, word_vec![1], word_vec![1], abs_positive);
        correct_operation!(abs, word_vec![-1], word_vec![1], abs_negative);

        // Logic
        correct_operation!(equal, word_vec![5, 4], word_vec![false], not_eq);
        correct_operation!(equal, word_vec![4, 4], word_vec![true]);
        correct_operation!(gt, word_vec![4, 4], word_vec![false], gt_equal);
        correct_operation!(gt, word_vec![5, 4], word_vec![false], gt_smaller);
        correct_operation!(gt, word_vec![4, 5], word_vec![true], gt_bigger);
        correct_operation!(lt, word_vec![4, 4], word_vec![false], lt_equal);
        correct_operation!(lt, word_vec![5, 4], word_vec![true], lt_smaller);
        correct_operation!(lt, word_vec![4, 5], word_vec![false], lt_bigger);
        correct_operation!(and, word_vec![true, true], word_vec![true], and_true_true);
        correct_operation!(
            and,
            word_vec![false, true],
            word_vec![false],
            and_false_true
        );
        correct_operation!(
            and,
            word_vec![false, false],
            word_vec![false],
            and_false_false
        );
        correct_operation!(or, word_vec![true, true], word_vec![true], or_true_true);
        correct_operation!(or, word_vec![false, true], word_vec![true], or_false_true);
        correct_operation!(
            or,
            word_vec![false, false],
            word_vec![false],
            or_false_false
        );
        correct_operation!(not, word_vec![true], word_vec![false], not_true);
        correct_operation!(not, word_vec![false], word_vec![true], not_false);
    }

    mod error {
        use super::*;
        use VMError::{NotEnoughtValues, TypeMismatch};

        macro_rules! errorneus_operation {
            ($method:ident, $stack:expr, $expected_err: expr) => {
                #[test]
                fn $method() {
                    let mut mem = unsafe { Memory::from_raw_parts($stack, 0, 0, 0) };

                    assert_eq!(mem.$method().unwrap_err(), $expected_err);
                }
            };
            ($method:ident, $stack:expr, $expected_err: expr, $name: ident) => {
                #[test]
                fn $name() {
                    let mut mem = unsafe { Memory::from_raw_parts($stack, 0, 0, 0) };

                    assert_eq!(mem.$method().unwrap_err(), $expected_err);
                }
            };
        }

        mod type_mismatch {
            use super::*;
            use pretty_assertions::assert_eq;

            // Arithmetic
            errorneus_operation!(add, word_vec![1, true], TypeMismatch);
            errorneus_operation!(sub, word_vec![1, true], TypeMismatch);
            errorneus_operation!(mul, word_vec![1, true], TypeMismatch);
            errorneus_operation!(div, word_vec![1, true], TypeMismatch);
            errorneus_operation!(rem, word_vec![1, true], TypeMismatch);
            errorneus_operation!(neg, word_vec![1, true], TypeMismatch);
            errorneus_operation!(inc, word_vec![1, true], TypeMismatch);
            errorneus_operation!(dec, word_vec![1, true], TypeMismatch);
            errorneus_operation!(abs, word_vec![1, true], TypeMismatch);

            // Logic
            errorneus_operation!(equal, word_vec![1, true], TypeMismatch);
            errorneus_operation!(gt, word_vec![1, true], TypeMismatch);
            errorneus_operation!(lt, word_vec![1, true], TypeMismatch);
            errorneus_operation!(and, word_vec![1, true], TypeMismatch);
            errorneus_operation!(or, word_vec![1, true], TypeMismatch);
            errorneus_operation!(not, word_vec![1], TypeMismatch);
        }

        mod not_enought_values {
            use super::*;
            use pretty_assertions::assert_eq;

            // Arithmetic
            errorneus_operation!(add, word_vec![1], NotEnoughtValues);
            errorneus_operation!(sub, word_vec![1], NotEnoughtValues);
            errorneus_operation!(mul, word_vec![1], NotEnoughtValues);
            errorneus_operation!(div, word_vec![1], NotEnoughtValues);
            errorneus_operation!(rem, word_vec![1], NotEnoughtValues);
            errorneus_operation!(neg, word_vec![], NotEnoughtValues);
            errorneus_operation!(inc, word_vec![], NotEnoughtValues);
            errorneus_operation!(dec, word_vec![], NotEnoughtValues);
            errorneus_operation!(abs, word_vec![], NotEnoughtValues);

            // Logic
            errorneus_operation!(equal, word_vec![1], NotEnoughtValues);
            errorneus_operation!(gt, word_vec![1], NotEnoughtValues);
            errorneus_operation!(lt, word_vec![1], NotEnoughtValues);
            errorneus_operation!(and, word_vec![true], NotEnoughtValues);
            errorneus_operation!(or, word_vec![true], NotEnoughtValues);
            errorneus_operation!(not, word_vec![], NotEnoughtValues);
        }
    }

    mod data_flow {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn push_external_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 0) };
            mem.push_external(Word::Numeric(0)).unwrap();

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![0], 0, 0, 1) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn pop_external_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6], 0, 0, 0) };

            assert_eq!(mem.pop_external(), Ok(Word::Numeric(6)));
            assert_eq!(mem.pop_external(), Ok(Word::Numeric(2)));
            assert_eq!(mem.pop_external(), Err(VMError::NotEnoughtValues));

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 2) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn pop_external_no_inc() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6], 0, 0, 0) };

            assert_eq!(mem.pop_external_no_pc_inc(), Ok(Word::Numeric(6)));
            assert_eq!(mem.pop_external_no_pc_inc(), Ok(Word::Numeric(2)));
            assert_eq!(mem.pop_external_no_pc_inc(), Err(VMError::NotEnoughtValues));

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 0) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn push_local_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8], 1, 0, 0) };

            mem.push_local(0).unwrap();
            mem.push_local(1).unwrap();

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8, 6, 8], 1, 0, 2) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn pop_local_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8, 16,], 0, 0, 0) };

            mem.pop_local(0).unwrap();
            mem.pop_local(1).unwrap();

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![16, 8], 0, 0, 2) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn push_argument_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8], 0, 1, 0) };

            mem.push_argument(0).unwrap();
            mem.push_argument(1).unwrap();

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8, 6, 8], 0, 1, 2) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn pop_argument_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8, 16,], 0, 0, 0) };

            mem.pop_argument(0).unwrap();
            mem.pop_argument(1).unwrap();

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![16, 8], 0, 0, 2) };

            assert_eq!(mem, mem_expected);
        }
    }

    mod control_flow {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn call() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, true], 0, 1, 4) };

            mem.call(16, 2).unwrap();

            let mem_expected =
                unsafe { Memory::from_raw_parts(word_vec![2, true, 5, 0, 1], 5, 0, 16) };

            assert_eq!(mem, mem_expected);
        }
        #[test]
        fn call_no_args() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 4) };

            mem.call(16, 0).unwrap();

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![5, 0, 0], 3, 0, 16) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn fn_return() {
            let mut mem =
                unsafe { Memory::from_raw_parts(word_vec![2, true, 5, 0, 1, false], 5, 0, 16) };

            mem.fn_return().unwrap();

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![false], 0, 1, 5) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn function() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, true], 2, 0, 16) };

            mem.function(3).unwrap();

            let mem_expected =
                unsafe { Memory::from_raw_parts(word_vec![2, true, 0, 0, 0], 2, 0, 17) };

            assert_eq!(mem, mem_expected);
        }

        mod ifjmp {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn conditional_jump_successful() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![true], 0, 0, 0) };

                mem.ifjmp(10).unwrap();

                let mem_expected = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 10) };

                assert_eq!(mem, mem_expected);
            }

            #[test]
            fn conditional_jump_unsuccessful() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![false], 0, 0, 0) };

                mem.ifjmp(10).unwrap();

                let mem_expected = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 1) };

                assert_eq!(mem, mem_expected);
            }

            #[test]
            fn type_mismatch() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0) };
                assert_eq!(mem.ifjmp(10), Err(VMError::TypeMismatch));
            }

            #[test]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0) };
                assert_eq!(mem.ifjmp(10), Err(VMError::NotEnoughtValues));
            }
        }
    }
}
