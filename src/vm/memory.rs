use crate::word::ConversionError;
use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryFrom;
use thiserror::Error;

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

#[derive(Error, Copy, Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum Error {
    #[error("VM halted")]
    Halt,
    #[error("Not enough values on the stack")]
    NotEnoughValues,
    #[error("Type mismatch")]
    TypeMismatch,
    #[error("Attempted to convert negative value to unsigned")]
    NegativeUnsignedValue,
    #[error("Attempted to access non-existent local variable")]
    LocalVarOutOfBounds,
    #[error("Attempted to access non-existent argument")]
    ArgumentOutOfBounds,
    #[error("CardType index is out of bounds")]
    NoSuchType,
    #[error("Acsess violation: attempted to access not-readable memory region")]
    AccessViolation,
    #[error("MemoryRegion index is out of bounds")]
    NoSuchRegion,
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
mod tests;
