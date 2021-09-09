use crate::card::{Card, CardType};
use crate::vm::Word;

#[derive(Debug)]
pub struct Board {
    card_types: Vec<CardType>,
    cards: Vec<Card>,
    attrs: Vec<Word>,
}

impl Board {
    pub fn get_attr_by_index(&self, index: usize) -> Word {
        self.attrs[index]
    }

    pub fn set_attr_by_index(&mut self, attr: Word, index: usize) {
        self.attrs[index] = attr;
    }

    pub fn check_attr_index(&self, index: usize) -> Result<(), ()> {
        if self.attrs.len() >= index {
            Ok(())
        } else {
            Err(())
        }
    }
}
