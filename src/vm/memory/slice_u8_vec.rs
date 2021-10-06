use crate::word::Word;
use std::{mem, slice};
use std::ops::Index;
use tinyvec::SliceVec;
use std::convert::TryInto;


#[derive(Eq, PartialEq, Debug)]
pub struct InternalStack<'a>(SliceVec<'a, InternalWord>);

type InternalWord = [u8; 5];

impl<'a> InternalStack<'a> {
    pub fn from_u8_slice(slice: &'a mut [u8], new_len: usize) -> Self {
        let len = slice.len();
        let element_size = mem::size_of::<InternalWord>();
        // Make sure we are going to read a full chunk of stuff
        assert_eq!(len % element_size, 0);

        // Remainer will always be empty, because we've already checked slice length
        let (new_slice, _) = slice.as_chunks_mut();

        InternalStack(SliceVec::<'a, InternalWord>::from_slice_len(
                new_slice, new_len,
        ))
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn pop(&mut self) -> Option<Word> {
        let internal_word = self.0.pop();
        internal_word.map(|word| {
            let descriminant = word[0];
            match descriminant {
                0 => {
                    let bool_data = word[1];
                    match bool_data {
                        0 => Word::Boolean(false),
                        1 => Word::Boolean(true),
                        _ => panic!("Memory corrupted!"),
                    }
                }
                1 => {
                    Word::Numeric(i32::from_le_bytes(word[1..].try_into().unwrap()))
                }
                _ => panic!("Memory corrupted!"),
            }
        })
    }

    pub fn push(&mut self, word: Word) {
        let internal_word = match word {
            Word::Boolean(false) => [0,0,0,0,0],
            Word::Boolean(true) => [0,1,0,0,0],
            Word::Numeric(val) => {
                let mut arr = [1,0,0,0,0];
                arr[1..].clone_from_slice(&val.to_le_bytes());
                arr
            },
        };
        self.0.push(internal_word);
    }
}

#[cfg(test)]
mod internal_stack_tests {
    use super::*;
    use crate::word_vec;
    use tinyvec::array_vec;
    use test_case::test_case;
    use std::iter;

    #[test]
    fn bool_false() {
        let mut raw_data = vec![0, 0, 0, 0, 0];

        let mut stack = InternalStack::from_u8_slice(&mut raw_data, 1);
        assert_eq!(stack.pop(), Some(Word::Boolean(false)));
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn bool_true() {
        let mut raw_data = vec![0, 1, 0, 0, 0];

        let mut stack = InternalStack::from_u8_slice(&mut raw_data, 1);
        assert_eq!(stack.pop(), Some(Word::Boolean(true)));
        assert_eq!(stack.len(), 0);
    }

    #[test]
    #[should_panic]
    fn bool_corrupted() {
        let mut raw_data = vec![0, 2, 0, 0, 0];

        let mut stack = InternalStack::from_u8_slice(&mut raw_data, 1);
        assert_eq!(stack.pop(), Some(Word::Boolean(true)));
    }

    #[test_case(0)]
    #[test_case(1)]
    #[test_case(i32::MIN)]
    #[test_case(i32::MAX)]
    fn numeric(num: i32) {
        let mut raw_data = vec![1];
        raw_data.extend(num.to_le_bytes());

        let mut stack = InternalStack::from_u8_slice(&mut raw_data, 1);
        assert_eq!(stack.pop(), Some(Word::Numeric(num)));
        assert_eq!(stack.len(), 0);
    }

    #[test_case(Word::Numeric(0))]
    #[test_case(Word::Numeric(1))]
    #[test_case(Word::Numeric(i32::MIN))]
    #[test_case(Word::Numeric(i32::MAX))]
    #[test_case(Word::Boolean(true))]
    #[test_case(Word::Boolean(false))]
    fn push_pop_equivalence(word: Word) {
        let mut raw_data: Vec<u8> = iter::repeat(0).take(mem::size_of::<InternalWord>()).collect();
        let mut stack = InternalStack::from_u8_slice(&mut raw_data, 0);
        stack.push(word);
        assert_eq!(stack.pop().unwrap(), word);
        
    }

}
