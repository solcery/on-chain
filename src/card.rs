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

const TYPE_ATTRS_VEC_SIZE: usize = 32;
type TypeAttrs = ArrayVec<[Word; TYPE_ATTRS_VEC_SIZE]>;

#[derive(Debug)]
pub struct CardType {
    pub id: u32,
    pub attrs: TypeAttrs,
}
impl Default for CardType {
    fn default() -> Self {
        CardType {
            id: 0,
            attrs: TypeAttrs::new(),
        }
    }
}
