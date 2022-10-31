use crate::on_chain_types::word::{ConversionError, Word};
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryFrom;

#[derive(Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Memory {
    stack: Vec<Word>,
    lcl: usize,
    arg: usize,
    pc: usize,
    n_args: usize,
    n_locals: usize,
}

macro_rules! two_nums_method {
    ($name:ident, $first:ident, $second:ident, $op:expr) => {
        pub fn $name(&mut self) -> Result<(), Error> {
            if self.lcl + self.n_locals + 2 <= self.stack.len() {
                let first_word = self.stack.pop();
                let second_word = self.stack.pop();
                match (first_word, second_word) {
                    (Some(Word::Numeric($first)), Some(Word::Numeric($second))) => {
                        self.stack.push(Word::from($op));
                        self.pc += 1;
                        Ok(())
                    }
                    (Some(Word::Boolean(_)), _) | (_, Some(Word::Boolean(_))) => {
                        Err(Error::TypeMismatch)
                    }
                    _ => unreachable!(),
                }
            } else {
                Err(Error::NotEnoughValues)
            }
        }
    };
}

macro_rules! one_num_method {
    ($name:ident, $var_ident: ident, $op:expr) => {
        pub fn $name(&mut self) -> Result<(), Error> {
            if self.lcl + self.n_locals < self.stack.len() {
                let value = self.stack.pop();
                match value {
                    Some(Word::Numeric($var_ident)) => {
                        self.stack.push(Word::from($op));
                        self.pc += 1;
                        Ok(())
                    }
                    Some(Word::Boolean(_)) => Err(Error::TypeMismatch),
                    _ => unreachable!(),
                }
            } else {
                Err(Error::NotEnoughValues)
            }
        }
    };
}

macro_rules! two_bools_method {
    ($name:ident, $first:ident, $second:ident, $op:expr) => {
        pub fn $name(&mut self) -> Result<(), Error> {
            if self.lcl + self.n_locals + 2 <= self.stack.len() {
                let first_word = self.stack.pop();
                let second_word = self.stack.pop();
                match (first_word, second_word) {
                    (Some(Word::Boolean($first)), Some(Word::Boolean($second))) => {
                        self.stack.push(Word::Boolean($op));
                        self.pc += 1;
                        Ok(())
                    }
                    (Some(Word::Numeric(_)), _) | (_, Some(Word::Numeric(_))) => {
                        Err(Error::TypeMismatch)
                    }
                    _ => unreachable!(),
                }
            } else {
                Err(Error::NotEnoughValues)
            }
        }
    };
}

macro_rules! push_variable {
    ($name:ident, $n_variables:ident, $variable_kind:ident, $err:ident) => {
        pub fn $name(&mut self, index: usize) -> Result<(), Error> {
            if index < self.$n_variables {
                let value = self.stack[self.$variable_kind + index];
                self.stack.push(value);
                self.pc += 1;
                Ok(())
            } else {
                Err(Error::$err)
            }
        }
    };
}

macro_rules! pop_variable {
    ($name:ident, $n_variables:ident, $variable_kind:ident, $err:ident) => {
        pub fn $name(&mut self, index: usize) -> Result<(), Error> {
            if index < self.$n_variables {
                if self.lcl + self.n_locals < self.stack.len() {
                    match self.stack.pop() {
                        Some(value) => {
                            self.stack[self.$variable_kind + index] = value;
                            self.pc += 1;
                            Ok(())
                        }
                        _ => unreachable!(),
                    }
                } else {
                    Err(Error::NotEnoughValues)
                }
            } else {
                Err(Error::$err)
            }
        }
    };
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
            n_args: 0,
            n_locals: 0,
        }
    }

    pub fn pc(&self) -> usize {
        self.pc
    }

    pub fn jmp(&mut self, address: usize) -> Result<(), Error> {
        self.pc = address;
        Ok(())
    }

    pub fn ifjmp(&mut self, address: usize) -> Result<(), Error> {
        if self.lcl + self.n_locals < self.stack.len() {
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
                Some(Word::Numeric(_)) => Err(Error::TypeMismatch),
                _ => unreachable!(),
            }
        } else {
            Err(Error::NotEnoughValues)
        }
    }

    two_nums_method!(add, x, y, x + y);
    two_nums_method!(sub, x, y, x - y);
    two_nums_method!(mul, x, y, x * y);
    two_nums_method!(div, x, y, x / y);
    two_nums_method!(rem, x, y, x % y);
    two_nums_method!(equal, x, y, x == y);
    two_nums_method!(gt, x, y, x > y);
    two_nums_method!(lt, x, y, x < y);

    one_num_method!(neg, x, -x);
    one_num_method!(inc, x, x + 1);
    one_num_method!(dec, x, x - 1);
    one_num_method!(abs, x, x.abs());

    two_bools_method!(and, x, y, x && y);
    two_bools_method!(or, x, y, x || y);

    push_variable!(push_local, n_locals, lcl, LocalVarOutOfBounds);
    push_variable!(push_argument, n_args, arg, ArgumentOutOfBounds);

    pop_variable!(pop_local, n_locals, lcl, LocalVarOutOfBounds);
    pop_variable!(pop_argument, n_args, arg, ArgumentOutOfBounds);

    pub fn push_external(&mut self, value: Word) -> Result<(), Error> {
        self.stack.push(value);
        self.pc += 1;
        Ok(())
    }

    pub fn pop_external(&mut self) -> Result<Word, Error> {
        if self.lcl + self.n_locals < self.stack.len() {
            match self.stack.pop() {
                Some(value) => {
                    self.pc += 1;
                    Ok(value)
                }
                _ => unreachable!(),
            }
        } else {
            Err(Error::NotEnoughValues)
        }
    }

    pub fn pop_external_no_pc_inc(&mut self) -> Result<Word, Error> {
        if self.lcl + self.n_locals < self.stack.len() {
            self.stack.pop().ok_or(Error::NotEnoughValues)
        } else {
            Err(Error::NotEnoughValues)
        }
    }

    pub fn not(&mut self) -> Result<(), Error> {
        if self.lcl + self.n_locals < self.stack.len() {
            let value = self.stack.pop();
            match value {
                Some(Word::Boolean(x)) => {
                    self.stack.push(Word::Boolean(!x));
                    self.pc += 1;
                    Ok(())
                }
                Some(Word::Numeric(_)) => Err(Error::TypeMismatch),
                _ => unreachable!(),
            }
        } else {
            Err(Error::NotEnoughValues)
        }
    }

    pub fn call(&mut self, address: usize, n_args: usize) -> Result<(), Error> {
        let return_address = self.pc + 1;
        self.stack.push(Word::Numeric(return_address as i32));
        self.stack.push(Word::Numeric(self.lcl as i32));
        self.stack.push(Word::Numeric(self.arg as i32));
        self.stack.push(Word::Numeric(self.n_args as i32));
        self.stack.push(Word::Numeric(self.n_locals as i32));
        self.lcl = self.stack.len();
        self.arg = self.stack.len() - n_args - 5;
        self.n_args = n_args;
        self.pc = address;
        Ok(())
    }

    pub fn function(&mut self, n_locals: usize) -> Result<(), Error> {
        self.n_locals = n_locals;
        for _ in 0..n_locals {
            self.stack.push(Word::Numeric(0));
        }
        self.pc += 1;
        Ok(())
    }

    pub fn fn_return(&mut self) -> Result<(), Error> {
        if self.lcl + self.n_locals < self.stack.len() {
            let frame = self.lcl;
            let return_address = usize::try_from(self.stack[frame - 5])?;
            let previous_lcl = usize::try_from(self.stack[frame - 4])?;
            let previous_arg = usize::try_from(self.stack[frame - 3])?;
            let previous_n_args = usize::try_from(self.stack[frame - 2])?;
            let previous_n_locals = usize::try_from(self.stack[frame - 1])?;
            let return_value = self.stack.pop().ok_or(Error::NotEnoughValues)?;

            self.stack.truncate(self.arg);
            self.stack.push(return_value);
            self.lcl = previous_lcl;
            self.arg = previous_arg;
            self.n_args = previous_n_args;
            self.n_locals = previous_n_locals;
            self.pc = return_address;
            Ok(())
        } else {
            Err(Error::NotEnoughValues)
        }
    }

    pub fn return_void(&mut self) -> Result<(), Error> {
        let frame = self.lcl;
        let return_address = usize::try_from(self.stack[frame - 5])?;
        let previous_lcl = usize::try_from(self.stack[frame - 4])?;
        let previous_arg = usize::try_from(self.stack[frame - 3])?;
        let previous_n_args = usize::try_from(self.stack[frame - 2])?;
        let previous_n_locals = usize::try_from(self.stack[frame - 1])?;

        self.stack.truncate(self.arg);
        self.lcl = previous_lcl;
        self.arg = previous_arg;
        self.n_args = previous_n_args;
        self.n_locals = previous_n_locals;
        self.pc = return_address;
        Ok(())
    }

    pub fn args(&self) -> Vec<Word> {
        self.stack[self.arg..self.arg + self.n_args].to_vec()
    }

    #[cfg(test)]
    pub unsafe fn from_raw_parts(
        stack: Vec<Word>,
        lcl: usize,
        arg: usize,
        pc: usize,
        n_args: usize,
        n_locals: usize,
    ) -> Self {
        if arg * n_args != 0 {
            assert_eq!(arg + n_args + 5, lcl);
        }
        assert!(lcl + n_locals <= stack.len());
        Self {
            stack,
            lcl,
            arg,
            pc,
            n_args,
            n_locals,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum Error {
    Halt,
    NotEnoughValues,
    TypeMismatch,
    NegativeUnsignedValue,
    LocalVarOutOfBounds,
    ArgumentOutOfBounds,
    NoSuchType,
}

impl From<ConversionError> for Error {
    fn from(err: ConversionError) -> Self {
        match err {
            ConversionError::TypeMismatch => Self::TypeMismatch,
            ConversionError::NegativeUnsignedValue => Self::NegativeUnsignedValue,
        }
    }
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
                    let mut mem = unsafe { Memory::from_raw_parts($stack, 0, 0, 0, 0, 0) };
                    mem.$method().unwrap();
                    let mem_expected =
                        unsafe { Memory::from_raw_parts($expected_stack, 0, 0, 1, 0, 0) };
                    assert_eq!(mem, mem_expected);
                }
            };
            ($method:ident, $stack:expr, $expected_stack: expr, $name: ident) => {
                #[test]
                fn $name() {
                    let mut mem = unsafe { Memory::from_raw_parts($stack, 0, 0, 0, 0, 0) };
                    mem.$method().unwrap();
                    let mem_expected =
                        unsafe { Memory::from_raw_parts($expected_stack, 0, 0, 1, 0, 0) };
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
        use Error::{NotEnoughValues, TypeMismatch};

        macro_rules! erroneous_operation {
            ($method:ident, $stack:expr, $expected_err: expr) => {
                #[test]
                fn $method() {
                    let mut mem = unsafe { Memory::from_raw_parts($stack, 0, 0, 0, 0, 0) };

                    assert_eq!(mem.$method().unwrap_err(), $expected_err);
                }
            };
            ($method:ident, $stack:expr, $expected_err: expr, $name: ident) => {
                #[test]
                fn $name() {
                    let mut mem = unsafe { Memory::from_raw_parts($stack, 0, 0, 0, 0, 0) };

                    assert_eq!(mem.$method().unwrap_err(), $expected_err);
                }
            };
        }

        mod type_mismatch {
            use super::*;
            use pretty_assertions::assert_eq;

            // Arithmetic
            erroneous_operation!(add, word_vec![1, true], TypeMismatch);
            erroneous_operation!(sub, word_vec![1, true], TypeMismatch);
            erroneous_operation!(mul, word_vec![1, true], TypeMismatch);
            erroneous_operation!(div, word_vec![1, true], TypeMismatch);
            erroneous_operation!(rem, word_vec![1, true], TypeMismatch);
            erroneous_operation!(neg, word_vec![1, true], TypeMismatch);
            erroneous_operation!(inc, word_vec![1, true], TypeMismatch);
            erroneous_operation!(dec, word_vec![1, true], TypeMismatch);
            erroneous_operation!(abs, word_vec![1, true], TypeMismatch);

            // Logic
            erroneous_operation!(equal, word_vec![1, true], TypeMismatch);
            erroneous_operation!(gt, word_vec![1, true], TypeMismatch);
            erroneous_operation!(lt, word_vec![1, true], TypeMismatch);
            erroneous_operation!(and, word_vec![1, true], TypeMismatch);
            erroneous_operation!(or, word_vec![1, true], TypeMismatch);
            erroneous_operation!(not, word_vec![1], TypeMismatch);
        }

        mod not_enough_values {
            use super::*;
            use pretty_assertions::assert_eq;

            // Arithmetic
            erroneous_operation!(add, word_vec![1], NotEnoughValues);
            erroneous_operation!(sub, word_vec![1], NotEnoughValues);
            erroneous_operation!(mul, word_vec![1], NotEnoughValues);
            erroneous_operation!(div, word_vec![1], NotEnoughValues);
            erroneous_operation!(rem, word_vec![1], NotEnoughValues);
            erroneous_operation!(neg, word_vec![], NotEnoughValues);
            erroneous_operation!(inc, word_vec![], NotEnoughValues);
            erroneous_operation!(dec, word_vec![], NotEnoughValues);
            erroneous_operation!(abs, word_vec![], NotEnoughValues);

            // Logic
            erroneous_operation!(equal, word_vec![1], NotEnoughValues);
            erroneous_operation!(gt, word_vec![1], NotEnoughValues);
            erroneous_operation!(lt, word_vec![1], NotEnoughValues);
            erroneous_operation!(and, word_vec![true], NotEnoughValues);
            erroneous_operation!(or, word_vec![true], NotEnoughValues);
            erroneous_operation!(not, word_vec![], NotEnoughValues);
        }
    }

    mod data_flow {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn push_external_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 0, 0, 0) };
            mem.push_external(Word::Numeric(0)).unwrap();

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![0], 0, 0, 1, 0, 0) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn pop_external_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6], 0, 0, 0, 0, 0) };

            assert_eq!(mem.pop_external(), Ok(Word::Numeric(6)));
            assert_eq!(mem.pop_external(), Ok(Word::Numeric(2)));
            assert_eq!(mem.pop_external(), Err(Error::NotEnoughValues));

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 2, 0, 0) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn pop_external_no_inc() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6], 0, 0, 0, 0, 0) };

            assert_eq!(mem.pop_external_no_pc_inc(), Ok(Word::Numeric(6)));
            assert_eq!(mem.pop_external_no_pc_inc(), Ok(Word::Numeric(2)));
            assert_eq!(mem.pop_external_no_pc_inc(), Err(Error::NotEnoughValues));

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 0, 0, 0) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn push_local_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8], 1, 0, 0, 0, 2) };

            mem.push_local(0).unwrap();
            mem.push_local(1).unwrap();

            let mem_expected =
                unsafe { Memory::from_raw_parts(word_vec![2, 6, 8, 6, 8], 1, 0, 2, 0, 2) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn pop_local_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8, 16,], 0, 0, 0, 0, 2) };

            mem.pop_local(0).unwrap();
            mem.pop_local(1).unwrap();

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![16, 8], 0, 0, 2, 0, 2) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn push_argument_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8], 0, 0, 0, 3, 0) };

            mem.push_argument(1).unwrap();
            mem.push_argument(2).unwrap();

            let mem_expected =
                unsafe { Memory::from_raw_parts(word_vec![2, 6, 8, 6, 8], 0, 0, 2, 3, 0) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn pop_argument_data() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, 6, 8, 16,], 0, 0, 0, 2, 0) };

            mem.pop_argument(0).unwrap();
            mem.pop_argument(1).unwrap();

            let mem_expected = unsafe { Memory::from_raw_parts(word_vec![16, 8], 0, 0, 2, 2, 0) };

            assert_eq!(mem, mem_expected);
        }
    }

    mod control_flow {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn call() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, true], 0, 1, 4, 0, 0) };

            mem.call(16, 2).unwrap();

            let mem_expected = unsafe {
                Memory::from_raw_parts(word_vec![2, true, 5, 0, 1, 0, 0], 7, 0, 16, 2, 0)
            };

            assert_eq!(mem, mem_expected);
        }
        #[test]
        fn call_no_args() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 4, 0, 0) };

            mem.call(16, 0).unwrap();

            let mem_expected =
                unsafe { Memory::from_raw_parts(word_vec![5, 0, 0, 0, 0], 5, 0, 16, 0, 0) };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn fn_return() {
            let mut mem = unsafe {
                Memory::from_raw_parts(
                    word_vec![
                        // locals of the previous stack frame
                        1,     // prev_local 0
                        1,     // prev_local 1
                        1,     // prev_local 2
                        2,     // arg 0
                        true,  // arg 1
                        5,     // return address
                        0,     // prev_lcl
                        0,     // prev_arg
                        0,     // prev_n_args
                        3,     // prev_n_locals
                        3,     // loc 0
                        4,     // loc 1
                        5,     // loc 2
                        6,     // loc 3
                        false, // return value of the function
                    ],
                    10, // lcl
                    3,  // arg
                    16, // pc
                    2,  // n_args
                    4,  // n_locals
                )
            };

            mem.fn_return().unwrap();

            let mem_expected = unsafe {
                Memory::from_raw_parts(
                    word_vec![
                        1,     // local 0
                        1,     // local 1
                        1,     // local 2
                        false, // return value of the function
                    ],
                    0, // arg
                    0, // lcl
                    5, // pc
                    0, // n_args
                    3, // n_locals
                )
            };

            assert_eq!(mem, mem_expected);
        }

        #[test]
        fn function() {
            let mut mem = unsafe { Memory::from_raw_parts(word_vec![2, true], 2, 0, 16, 0, 0) };

            mem.function(3).unwrap();

            let mem_expected =
                unsafe { Memory::from_raw_parts(word_vec![2, true, 0, 0, 0], 2, 0, 17, 0, 3) };

            assert_eq!(mem, mem_expected);
        }

        mod ifjmp {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn conditional_jump_successful() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![true], 0, 0, 0, 0, 0) };

                mem.ifjmp(10).unwrap();

                let mem_expected = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 10, 0, 0) };

                assert_eq!(mem, mem_expected);
            }

            #[test]
            fn conditional_jump_unsuccessful() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![false], 0, 0, 0, 0, 0) };

                mem.ifjmp(10).unwrap();

                let mem_expected = unsafe { Memory::from_raw_parts(word_vec![], 0, 0, 1, 0, 0) };

                assert_eq!(mem, mem_expected);
            }

            #[test]
            fn type_mismatch() {
                let mut mem = unsafe { Memory::from_raw_parts(word_vec![1], 0, 0, 0, 0, 0) };
                assert_eq!(mem.ifjmp(10), Err(Error::TypeMismatch));
            }

            #[test]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(Vec::<Word>::new(), 0, 0, 0, 0, 0) };
                assert_eq!(mem.ifjmp(10), Err(Error::NotEnoughValues));
            }
        }
    }
}
