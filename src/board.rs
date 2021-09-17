use crate::card::{Card, CardType};
use crate::word::Word;
use tinyvec::ArrayVec;

const DECK_SIZE: usize = 512;
type Deck = ArrayVec<[Card; DECK_SIZE]>;

const TYPEDECK_SIZE: usize = 256;
type TypeDeck = ArrayVec<[CardType; TYPEDECK_SIZE]>;

const ATTR_VEC_SIZE: usize = 128;
type AttrVec = ArrayVec<[Word; ATTR_VEC_SIZE]>;

#[derive(Debug)]
pub struct Board {
    card_types: TypeDeck,
    pub cards: Deck,
    attrs: AttrVec,
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
