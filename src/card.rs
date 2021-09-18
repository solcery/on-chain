use crate::word::Word;
use std::marker::Copy;
use tinyvec::ArrayVec;

const ATTRS_VEC_SIZE: usize = 32;
type Attrs = ArrayVec<[Word; ATTRS_VEC_SIZE]>;

#[derive(Debug, Clone, Copy)]
pub struct Card {
    id: u32,
    card_type: u32,
    pub attrs: Attrs,
}

impl Card {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn card_type(&self) -> u32 {
        self.card_type
    }
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
    id: u32,
    attrs: TypeAttrs,
}

impl CardType {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn get_attr_by_index(&self, index: usize) -> Word {
        self.attrs[index]
    }
}
impl Default for CardType {
    fn default() -> Self {
        CardType {
            id: 0,
            attrs: TypeAttrs::new(),
        }
    }
}
