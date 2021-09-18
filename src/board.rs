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
    pub card_types: TypeDeck,
    pub cards: Deck,
    pub attrs: AttrVec,
}
