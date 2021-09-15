use crate::card::{Card, CardType};
use crate::word::Word;
use tinyvec::ArrayVec;

const DECK_SIZE: usize = 512;
type Deck = ArrayVec<[Card; DECK_SIZE]>;

#[derive(Debug)]
pub struct Board {
    card_types: Vec<CardType>,
    cards: Deck,
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
