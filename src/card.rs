use crate::word::Word;
use std::marker::Copy;
use tinyvec::ArrayVec;

const ATTRS_VEC_SIZE: usize = 32;
type Attrs = ArrayVec<[Word; ATTRS_VEC_SIZE]>;

#[derive(Debug, Clone, Copy)]
pub struct Card {
    id: u32,
    card_type: u32,
    attrs: Attrs,
}

impl Default for Card {
    fn default() -> Self {
        Card {
            id: 0,
            card_type: 0,
            attrs: Attrs::new(),
        }
    }
}

const DATA_VEC_SIZE: usize = 32;
type Data = ArrayVec<[Word; DATA_VEC_SIZE]>;

#[derive(Debug)]
pub struct CardType {
    pub id: u32,
    pub data: Data,
}

impl CardType {}
