use crate::word::Word;
use std::{mem, slice};
use std::ops::Index;
use tinyvec::SliceVec;

#[derive(Eq, PartialEq, Debug)]
pub struct InternalStack<'a>(SliceVec<'a, InternalWord>);

#[derive(Eq, PartialEq, Debug)]
struct InternalWord(u32, i32);

impl Default for InternalWord {
    fn default() -> Self {
        InternalWord(0, 0)
    }
}

impl<'a> InternalStack<'a> {
    pub fn from_u8_slice(slice: &'a mut [u8], new_len: usize) -> Self {
        let len = slice.len();
        let element_size = mem::size_of::<InternalWord>();
        // Make sure we are going to read a full chunk of stuff
        assert_eq!(len % element_size, 0);

        let (head, body, tail) = unsafe { slice.align_to::<InternalWord>() };
        assert!(head.is_empty());
        assert!(tail.is_empty());

        let data = body.as_ptr();

        unsafe {
            // Don't allow the current slice to be dropped
            // (which would invalidate the memory)
            mem::forget(body);

            let new_slice =
                slice::from_raw_parts_mut(data as *mut InternalWord, len / element_size);

            InternalStack(SliceVec::<'a, InternalWord>::from_slice_len(
                new_slice, new_len,
            ))
        }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn pop(&mut self) -> Option<Word> {
        let internal_word = self.0.pop();
        internal_word.map(|word| match word {
            InternalWord(0, 0) => Word::Boolean(false),
            InternalWord(0, 1) => Word::Boolean(true),
            InternalWord(0, _) => panic!("Memory corrupted!"),
            InternalWord(1, i) => Word::Numeric(i),
            InternalWord(_, _) => panic!("Memory corrupted!"),
        })
    }

    pub fn push(&mut self, word: Word) {
        let internal_word = match word {
            Word::Boolean(false) => InternalWord(0, 0),
            Word::Boolean(true) => InternalWord(0, 1),
            Word::Numeric(val) => InternalWord(1, val),
        };
        self.0.push(internal_word);
    }
}

#[cfg(test)]
mod internal_stack_tests {
    use super::*;
    use crate::word_vec;
    use tinyvec::array_vec;

    #[test]
    fn alligned() {
        let mut raw_data = vec![0, 0, 0, 0, 0, 0, 0, 0];

        let mut stack = unsafe { InternalStack::from_u8_slice(&mut raw_data, 1) };
        assert_eq!(stack.pop(), Some(Word::Boolean(false)));
        assert_eq!(stack.len(), 0);
    }

    #[test]
    #[should_panic]
    fn unalligned() {
        let mut raw_data = vec![1, 0, 0, 0, 0, 0, 0, 0, 0];

        let mut stack = unsafe { InternalStack::from_u8_slice(&mut raw_data[1..], 1) };
        assert_eq!(stack.pop(), Some(Word::Boolean(false)));
        assert_eq!(stack.len(), 0);
    }
}
