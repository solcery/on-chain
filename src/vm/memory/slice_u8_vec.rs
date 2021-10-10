use crate::word::Word;
use std::convert::TryInto;
use std::fmt;
use std::mem;
use tinyvec::SliceVec;

type InternalWord = [u8; 5];

impl From<InternalWord> for Word {
    fn from(word: InternalWord) -> Self {
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
            1 => Word::Numeric(i32::from_le_bytes(word[1..].try_into().unwrap())),
            _ => panic!("Memory corrupted!"),
        }
    }
}

impl From<Word> for InternalWord {
    fn from(word: Word) -> Self {
        match word {
            Word::Boolean(false) => [0, 0, 0, 0, 0],
            Word::Boolean(true) => [0, 1, 0, 0, 0],
            Word::Numeric(val) => {
                let mut arr = [1, 0, 0, 0, 0];
                arr[1..].clone_from_slice(&val.to_le_bytes());
                arr
            }
        }
    }
}

#[derive(Eq, PartialEq)]
pub struct InternalStack<'a>(SliceVec<'a, InternalWord>);

impl<'a> fmt::Debug for InternalStack<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|word| Word::from(*word)))
            .finish()
    }
}

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
        self.0.pop().map(|word| Word::from(word))
    }

    pub fn push(&mut self, word: Word) {
        self.0.push(InternalWord::from(word));
    }

    pub fn word(&self, index: usize) -> Word {
        Word::from(self.0[index])
    }

    pub fn set_word(&mut self, index: usize, word: Word) {
        self.0[index] = InternalWord::from(word);
    }

    pub fn truncate(&mut self, len: usize) {
        self.0.truncate(len);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::word_vec;
    use test_case::test_case;

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
    fn to_from_equivalence(word: Word) {
        let new_word = InternalWord::from(word);
        assert_eq!(Word::from(new_word), word);
    }
}
