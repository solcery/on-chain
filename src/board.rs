use crate::card::{Card, CardType};
use crate::word::Word;
use std::convert::TryInto;
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
    pub attrs: AttrVec,
}
    pub fn card_type_count(&self) -> usize {
        self.card_types.len()
    }
    pub fn card_type_by_type_index(&self, type_index: usize) -> &CardType {
        &self.card_types[type_index]
    }
    pub fn card_type_by_type_id(&self, type_id: u32) -> Option<&CardType> {
        self.card_types
            .iter()
            .find(|card_type| card_type.id() == type_id)
    }
