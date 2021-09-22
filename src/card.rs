use crate::word::Word;
use tinyvec::ArrayVec;

const ATTRS_VEC_SIZE: usize = 32;
type Attrs = ArrayVec<[Word; ATTRS_VEC_SIZE]>;

#[derive(Debug)]
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

    pub fn new(id: u32, card_type: u32, attrs: Attrs) -> Self {
        Card {
            id,
            card_type,
            attrs,
        }
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
    init_card_attrs: Attrs,
}

impl CardType {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn attr_by_index(&self, index: usize) -> Word {
        self.attrs[index]
    }

    pub fn new(id: u32,  attrs: TypeAttrs,init_card_attrs: Attrs) -> Self {
        CardType {
            id,
            attrs,
            init_card_attrs,
        }
    }

    pub fn instantiate_card(&self, id: u32) -> Card {
        Card {
            id,
            card_type: self.id(),
            attrs: self.init_card_attrs.clone(),
        }
    }
}

impl Default for CardType {
    fn default() -> Self {
        CardType {
            id: 0,
            attrs: TypeAttrs::new(),
            init_card_attrs: Attrs::new(),
        }
    }
}
