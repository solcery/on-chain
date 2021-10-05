use crate::word::Word;
use std::convert::TryFrom;
use tinyvec::SliceVec;

const STACK_SIZE: usize = 512;
type Stack<'a> = SliceVec<'a, Word>;

mod slice_u8_vec;
use slice_u8_vec::InternalStack;

#[derive(Debug, Eq, PartialEq)]
pub struct Memory<'a> {
    stack: Stack<'a>,
    lcl: usize,
    arg: usize,
    pc: usize,
}

impl<'a> Memory<'a> {
    pub fn init_memory(arguments: &'a mut [Word], card_index: i32, action_index: i32) -> Self {
        let mut stack = Stack::from(arguments);
        stack.push(Word::Numeric(card_index));
        stack.push(Word::Numeric(action_index));

        Memory {
            stack,
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
            (Some(Word::Boolean(_)), _) | (_, Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to add boolean values.");
            }
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
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
            (Some(Word::Boolean(_)), _) | (_, Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to substract boolean values.");
            }
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
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
            (Some(Word::Boolean(_)), _) | (_, Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to multiply boolean values.");
            }
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
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
            (Some(Word::Boolean(_)), _) | (_, Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to divide boolean values.");
            }
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
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
            (Some(Word::Boolean(_)), _) | (_, Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to take the remainer of the boolean values.");
            }
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
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
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn inc(&mut self) {
        let value = self.stack.pop();
        match value {
            Some(Word::Numeric(x)) => {
                self.stack.push(Word::Numeric(x + 1));
                self.pc += 1;
            }
            Some(Word::Boolean(_)) => {
                panic!("Attempted to increment boolean value.");
            }
            None => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn dec(&mut self) {
        let value = self.stack.pop();
        match value {
            Some(Word::Numeric(x)) => {
                self.stack.push(Word::Numeric(x - 1));
                self.pc += 1;
            }
            Some(Word::Boolean(_)) => {
                panic!("Attempted to decrement boolean value.");
            }
            None => {
                panic!("Not enough values on the stack.");
            }
        }
    }

    pub fn abs(&mut self) {
        let value = self.stack.pop();
        match value {
            Some(Word::Numeric(x)) => {
                self.stack.push(Word::Numeric(x.abs()));
                self.pc += 1;
            }
            Some(Word::Boolean(_)) => {
                panic!("Attempted to find modulus of boolean value.");
            }
            None => {
                panic!("Not enough values on the stack.");
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

    pub fn pop_external_no_pc_inc(&mut self) -> Word {
        self.stack.pop().unwrap()
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

    pub fn equal(&mut self) {
        let first_word = self.stack.pop();
        let second_word = self.stack.pop();
        match (first_word, second_word) {
            (Some(Word::Numeric(x)), Some(Word::Numeric(y))) => {
                self.stack.push(Word::Boolean(x == y));
                self.pc += 1;
            }
            (Some(Word::Boolean(_)), Some(Word::Boolean(_))) => {
                panic!("Type mismatch: attempted to check boolean values for equality.");
            }
            (Some(_), Some(_)) => {
                panic!("Type mismatch: attempted to compare boolean to numerical.");
            }
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
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
                panic!("Type mismatch: attempted to check boolean values for equality.");
            }
            (Some(_), Some(_)) => {
                panic!("Type mismatch: attempted to compare boolean to numerical.");
            }
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
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
                panic!("Type mismatch: attempted to check boolean values for equality.");
            }
            (Some(_), Some(_)) => {
                panic!("Type mismatch: attempted to compare boolean to numerical.");
            }
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
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
                panic!("Type mismatch: attempted to AND numerical values.");
            }
            (Some(_), Some(_)) => {
                panic!("Type mismatch: attempted to AND boolean to numerical.");
            }
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
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
                panic!("Type mismatch: attempted to OR numerical values.");
            }
            (Some(_), Some(_)) => {
                panic!("Type mismatch: attempted to OR boolean to numerical.");
            }
            (_, None) | (None, _) => {
                panic!("Not enough values on the stack.");
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
                panic!("Not enough values on the stack.");
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

    #[cfg(test)]
    pub unsafe fn from_raw_parts(data: &mut [Word], lcl: usize, arg: usize, pc: usize) -> Memory {
        assert!(lcl <= data.len());
        assert!(arg <= data.len());
        Memory {
            stack: Stack::from(data),
            lcl,
            arg,
            pc,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::word_vec;
    use tinyvec::array_vec;

    mod arithmetic {
        use super::*;

        mod add {
            use super::*;

            #[test]
            fn numeric() {
                let mut stack = word_vec![1, 2];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.add();
                let mut stack_needed = word_vec![3];
                let mem_needed = unsafe { Memory::from_raw_parts(&mut stack_needed, 0, 0, 1) };
                pretty_assertions::assert_eq!(
                    mem,
                    mem_needed
                );
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to add boolean values.")]
            fn boolean_first() {
                let mut stack = word_vec![1, true];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.add();
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to add boolean values.")]
            fn boolean_second() {
                let mut stack = word_vec![true,1];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.add();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn one_value_on_the_stack() {
                let mut stack = word_vec![1];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.add();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                mem.add();
            }
        }

        mod sub {
            use super::*;

            #[test]
            fn numeric() {
                let mut stack = word_vec![1, 2];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.sub();
                let mut stack_needed = word_vec![1];
                let mem_needed = unsafe { Memory::from_raw_parts(&mut stack_needed, 0, 0, 1) };
                pretty_assertions::assert_eq!(
                    mem,
                    mem_needed
                );
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to substract boolean values.")]
            fn boolean_first() {
                let mut stack = word_vec![1, true];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.sub();
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to substract boolean values.")]
            fn boolean_second() {
                let mut stack = word_vec![true,1];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.sub();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn one_value_on_the_stack() {
                let mut stack = word_vec![1];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.sub();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                mem.sub();
            }
        }

        mod mul {
            use super::*;

            #[test]
            fn numeric() {
                let mut stack = word_vec![1, 2];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.mul();
                let mut stack_needed = word_vec![2];
                let mem_needed = unsafe { Memory::from_raw_parts(&mut stack_needed, 0, 0, 1) };
                pretty_assertions::assert_eq!(
                    mem,
                    mem_needed
                );
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to multiply boolean values.")]
            fn boolean_first() {
                let mut stack = word_vec![1, true];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.mul();
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to multiply boolean values.")]
            fn boolean_second() {
                let mut stack = word_vec![true,1];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.mul();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn one_value_on_the_stack() {
                let mut stack = word_vec![1];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.mul();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                mem.mul();
            }
        }

        mod div {
            use super::*;

            #[test]
            fn no_remainer() {
                let mut stack = word_vec![2, 6];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.div();
                let mut stack_needed = word_vec![3];
                let mem_needed = unsafe { Memory::from_raw_parts(&mut stack_needed, 0, 0, 1) };
                pretty_assertions::assert_eq!(
                    mem,
                    mem_needed
                );
            }

            #[test]
            fn remainer() {
                let mut stack = word_vec![2, 7];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.div();
                let mut stack_needed = word_vec![3];
                let mem_needed = unsafe { Memory::from_raw_parts(&mut stack_needed, 0, 0, 1) };
                pretty_assertions::assert_eq!(
                    mem,
                    mem_needed
                );
            }

            #[test]
            fn numeric() {
                let mut stack = word_vec![1, 2];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.div();
                let mut stack_needed = word_vec![2];
                let mem_needed = unsafe { Memory::from_raw_parts(&mut stack_needed, 0, 0, 1) };
                pretty_assertions::assert_eq!(
                    mem,
                    mem_needed
                );
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to divide boolean values.")]
            fn boolean_first() {
                let mut stack = word_vec![1, true];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.div();
            }

            #[test]
            #[should_panic(expected = "Type mismatch: attempted to divide boolean values.")]
            fn boolean_second() {
                let mut stack = word_vec![true,1];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.div();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn one_value_on_the_stack() {
                let mut stack = word_vec![1];
                let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                mem.div();
            }

            #[test]
            #[should_panic(expected = "Not enough values on the stack.")]
            fn empty_stack() {
                let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                mem.div();
            }
        }

        //mod rem {
            //use super::*;

            //#[test]
            //fn zero() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![2, 6], 0, 0, 0) };
                //mem.rem();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![0], 0, 0, 1) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //fn non_zero() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![3, 7], 0, 0, 0) };
                //mem.rem();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![1], 0, 0, 1) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //#[should_panic(
                //expected = "Type mismatch: attempted to take the remainer of the boolean values."
            //)]
            //fn boolean() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, true], 0, 0, 0) };
                //mem.rem();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn one_value_on_the_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1], 0, 0, 0) };
                //mem.rem();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn empty_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                //mem.rem();
            //}
        //}

        //mod neg {
            //use super::*;

            //#[test]
            //fn numeric() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, 2], 0, 0, 0) };
                //mem.neg();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //#[should_panic(expected = "Attempted to negate boolean value.")]
            //fn boolean() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, true], 0, 0, 0) };
                //mem.neg();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn empty_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                //mem.neg();
            //}
        //}

        //mod inc {
            //use super::*;

            //#[test]
            //fn numeric() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, 2], 0, 0, 0) };
                //mem.inc();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //#[should_panic(expected = "Attempted to increment boolean value.")]
            //fn boolean() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, true], 0, 0, 0) };
                //mem.inc();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn empty_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                //mem.inc();
            //}
        //}

        //mod dec {
            //use super::*;

            //#[test]
            //fn numeric() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, 2], 0, 0, 0) };
                //mem.dec();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //#[should_panic(expected = "Attempted to decrement boolean value.")]
            //fn boolean() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, true], 0, 0, 0) };
                //mem.dec();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn empty_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                //mem.dec();
            //}
        //}

        //mod abs {
            //use super::*;

            //#[test]
            //fn positive() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, 2], 0, 0, 0) };
                //mem.abs();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //fn negative() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, -2], 0, 0, 0) };
                //mem.abs();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //#[should_panic(expected = "Attempted to find modulus of boolean value.")]
            //fn boolean() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, true], 0, 0, 0) };
                //mem.abs();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn empty_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                //mem.abs();
            //}
        //}
    }

    //mod logic {
        //use super::*;

        //mod eq {
            //use super::*;

            //#[test]
            //fn equal() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![4, 4], 0, 0, 0) };
                //mem.equal();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //fn non_equal() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![5, 4], 0, 0, 0) };
                //mem.equal();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //#[should_panic(expected = "Type mismatch: attempted to compare boolean to numerical.")]
            //fn type_mismatch() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, true], 0, 0, 0) };
                //mem.equal();
            //}

            //#[test]
            //#[should_panic(
                //expected = "Type mismatch: attempted to check boolean values for equality."
            //)]
            //fn boolean() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![true, true], 0, 0, 0) };
                //mem.equal();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn one_value_on_the_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1], 0, 0, 0) };
                //mem.equal();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn empty_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                //mem.equal();
            //}
        //}

        //mod gt {
            //use super::*;

            //#[test]
            //fn equal() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![4, 4], 0, 0, 0) };
                //mem.gt();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //fn smaller() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![5, 4], 0, 0, 0) };
                //mem.gt();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //fn bigger() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![4, 5], 0, 0, 0) };
                //mem.gt();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //#[should_panic(expected = "Type mismatch: attempted to compare boolean to numerical.")]
            //fn type_mismatch() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, true], 0, 0, 0) };
                //mem.gt();
            //}

            //#[test]
            //#[should_panic(
                //expected = "Type mismatch: attempted to check boolean values for equality."
            //)]
            //fn boolean() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![true, true], 0, 0, 0) };
                //mem.gt();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn one_value_on_the_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1], 0, 0, 0) };
                //mem.gt();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn empty_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                //mem.gt();
            //}
        //}

        //mod lt {
            //use super::*;

            //#[test]
            //fn equal() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![4, 4], 0, 0, 0) };
                //mem.lt();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //fn smaller() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![5, 4], 0, 0, 0) };
                //mem.lt();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //fn bigger() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![4, 5], 0, 0, 0) };
                //mem.lt();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //#[should_panic(expected = "Type mismatch: attempted to compare boolean to numerical.")]
            //fn type_mismatch() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, true], 0, 0, 0) };
                //mem.lt();
            //}

            //#[test]
            //#[should_panic(
                //expected = "Type mismatch: attempted to check boolean values for equality."
            //)]
            //fn boolean() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![true, true], 0, 0, 0) };
                //mem.lt();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn one_value_on_the_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1], 0, 0, 0) };
                //mem.lt();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn empty_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                //mem.lt();
            //}
        //}

        //mod and {
            //use super::*;

            //#[test]
            //fn false_false() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![false, false], 0, 0, 0) };
                //mem.and();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //fn false_true() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![false, true], 0, 0, 0) };
                //mem.and();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //fn true_true() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![true, true], 0, 0, 0) };
                //mem.and();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //#[should_panic(expected = "Type mismatch: attempted to AND boolean to numerical.")]
            //fn type_mismatch() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, true], 0, 0, 0) };
                //mem.and();
            //}

            //#[test]
            //#[should_panic(expected = "Type mismatch: attempted to AND numerical values.")]
            //fn boolean() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, 2], 0, 0, 0) };
                //mem.and();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn one_value_on_the_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1], 0, 0, 0) };
                //mem.and();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn empty_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                //mem.and();
            //}
        //}

        //mod or {
            //use super::*;

            //#[test]
            //fn false_false() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![false, false], 0, 0, 0) };
                //mem.or();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //fn false_true() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![false, true], 0, 0, 0) };
                //mem.or();
                
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //fn true_true() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![true, true], 0, 0, 0) };
                //mem.or();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //#[should_panic(expected = "Type mismatch: attempted to OR boolean to numerical.")]
            //fn type_mismatch() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, true], 0, 0, 0) };
                //mem.or();
            //}

            //#[test]
            //#[should_panic(expected = "Type mismatch: attempted to OR numerical values.")]
            //fn boolean() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1, 2], 0, 0, 0) };
                //mem.or();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn one_value_on_the_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1], 0, 0, 0) };
                //mem.or();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn empty_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                //mem.or();
            //}
        //}

        //mod not {
            //use super::*;

            //#[test]
            //fn test_false() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![false], 0, 0, 0) };
                //mem.not();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //fn test_true() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![true], 0, 0, 0) };
                //mem.not();
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //#[should_panic(expected = "Attempted to NOT numerical value.")]
            //fn numerical() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![1], 0, 0, 0) };
                //mem.not();
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn empty_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                //mem.not();
            //}
        //}
    //}

    //mod data_flow {
        //use super::*;

        //#[test]
        //fn push_external_data() {
            //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![], 0, 0, 0) };
            //mem.push_external(Word::Numeric(0));
            //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
            //pretty_assertions::assert_eq!(
                //mem,
                //mem_needed
            //);
        //}

        //#[test]
        //fn pop_external_data() {
            //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![2, 6], 0, 0, 0) };
            //mem.pop_external();
            //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
            //pretty_assertions::assert_eq!(
                //mem,
                //mem_needed
            //);
        //}

        //#[test]
        //fn pop_external_no_inc() {
            //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![2, 6], 0, 0, 0) };
            //mem.pop_external_no_pc_inc();
            //mem.pop_external_no_pc_inc();
            //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
            //pretty_assertions::assert_eq!(
                //mem,
                //mem_needed
            //);
        //}

        //#[test]
        //fn push_local_data() {
            //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![2, 6, 8], 1, 0, 0) };
            //mem.push_local(0);
            //mem.push_local(1);

            //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
            //pretty_assertions::assert_eq!(
                //mem,
                //mem_needed
            //);
        //}

        //#[test]
        //fn pop_local_data() {
            //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![2, 6, 8, 16,], 0, 0, 0) };
            //mem.pop_local(0);
            //mem.pop_local(1);
            //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
            //pretty_assertions::assert_eq!(
                //mem,
                //mem_needed
            //);
        //}

        //#[test]
        //fn push_argument_data() {
            //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![2, 6, 8], 0, 1, 0) };
            //mem.push_argument(0);
            //mem.push_argument(1);
            //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
            //pretty_assertions::assert_eq!(
                //mem,
                //mem_needed
            //);
        //}

        //#[test]
        //fn pop_argument_data() {
            //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![2, 6, 8, 16,], 0, 0, 0) };
            //mem.pop_argument(0);
            //mem.pop_argument(1);

            //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![16,8], 0, 0, 2) };
            //pretty_assertions::assert_eq!(
                //mem,
                //mem_needed
            //);
        //}
    //}

    //mod control_flow {
        //use super::*;

        //#[test]
        //fn call() {
            //let mut mem = unsafe { Memory::from_raw_parts(&mut word_vec![2, true], 0, 1, 4) };
            //mem.call(16, 2);
            //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut word_vec![2, true, 5,0,1], 5, 0, 16) };
            //pretty_assertions::assert_eq!(
                //mem,
                //mem_needed
            //);
        //}

        //#[test]
        //fn fn_return() {
            //let mut stack = word_vec![2, true, 5, 0, 1, false];
            //let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 5, 0, 16) };
            //mem.fn_return();
            //let mut mem_needed =
                //unsafe { Memory::from_raw_parts(&mut word_vec![false], 0, 1, 5) };
            //pretty_assertions::assert_eq!(
                //mem,
                //mem_needed
            //);
        //}

        //#[test]
        //fn function() {
            //let mut stack = word_vec![2,true];
            //let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 2, 0, 16) };
            //mem.function(3);

            //let mut stack_needed = word_vec![2, true,0,0,0];
            //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut stack_needed, 2, 0, 17) };
            //pretty_assertions::assert_eq!(
                //mem,
                //mem_needed
            //);
        //}

        //mod ifjmp {
            //use super::*;
            //#[test]
            //fn conditional_jump_successful() {
                //let mut stack = word_vec![true];
                //let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                //mem.ifjmp(10);
                //let mut stack_needed = word_vec![];
                //let mut mem_needed = unsafe { Memory::from_raw_parts(&mut stack_needed, 0, 0, 10) };
                //pretty_assertions::assert_eq!(
                    //mem,
                    //mem_needed
                //);
            //}

            //#[test]
            //fn conditional_jump_unsuccessful() {
                //let mut stack = word_vec![false];
                //let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                //mem.ifjmp(10);
                //pretty_assertions::assert_eq!(mem.stack, SliceVec::<'_, Word>::from(&mut []));
                //pretty_assertions::assert_eq!(mem.pc, 1);
            //}

            //#[test]
            //#[should_panic(
                //expected = "Type mismatch: attempted to use numerical value in boolean condition."
            //)]
            //fn type_mismatch() {
                //let mut stack = word_vec![1];
                //let mut mem = unsafe { Memory::from_raw_parts(&mut stack, 0, 0, 0) };
                //mem.ifjmp(10);
            //}

            //#[test]
            //#[should_panic(expected = "Not enough values on the stack.")]
            //fn empty_stack() {
                //let mut mem = unsafe { Memory::from_raw_parts(&mut [], 0, 0, 0) };
                //mem.ifjmp(10);
            //}
        //}
    //}
}
