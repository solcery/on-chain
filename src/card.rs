use crate::word::Word;
use serde::{Deserialize, Serialize};
use tinyvec::ArrayVec;

const ATTRS_VEC_SIZE: usize = 2_usize.pow(4);
type Attrs = ArrayVec<[Word; ATTRS_VEC_SIZE]>;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Card {
    id: u32,
    card_type: u32,
    // На данный момент мы считаем все атрибуты статическими в том смысле, что у всех карт attrs[i]
    // имеет один и тот же игровой смысл.
    // Если мы оставляем это так, то имеет смысл сделать attrs приватным полем и сделать на него
    // сеттер таким образом, чтобы он не мог поменять num на bool
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

    pub unsafe fn from_raw_parts(id: u32, card_type: u32, attrs: Vec<Word>) -> Self {
        let mut card = Card {
            id,
            card_type,
            attrs: Attrs::new(),
        };
        card.attrs.fill(attrs);
        card
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

#[derive(Debug, Eq, PartialEq, Copy, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    address: u32,
    n_args: u32,
}

impl EntryPoint {
    pub fn address(&self) -> usize {
        self.address as usize
    }

    pub fn n_args(&self) -> usize {
        self.n_args as usize
    }

    pub unsafe fn from_raw_parts(address: u32, n_args: u32) -> EntryPoint {
        EntryPoint { address, n_args }
    }
}

impl Default for EntryPoint {
    fn default() -> Self {
        EntryPoint {
            address: 0,
            n_args: 0,
        }
    }
}

const TYPE_ATTRS_VEC_SIZE: usize = 2_usize.pow(4);
type TypeAttrs = ArrayVec<[Word; TYPE_ATTRS_VEC_SIZE]>;

const ENTRY_POINTS_VEC_SIZE: usize = 2_usize.pow(4);
type EntryPoints = ArrayVec<[EntryPoint; ENTRY_POINTS_VEC_SIZE]>;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CardType {
    id: u32,
    attrs: TypeAttrs,
    init_card_attrs: Attrs,
    action_entry_points: EntryPoints,
}

impl CardType {
    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn attr_by_index(&self, index: usize) -> Word {
        self.attrs[index]
    }

    pub fn new(
        id: u32,
        attrs: TypeAttrs,
        init_card_attrs: Attrs,
        action_entry_points: EntryPoints,
    ) -> Self {
        CardType {
            id,
            attrs,
            init_card_attrs,
            action_entry_points,
        }
    }

    pub fn instantiate_card(&self, id: u32) -> Card {
        Card {
            id,
            card_type: self.id(),
            attrs: self.init_card_attrs,
        }
    }

    pub fn action_entry_point(&self, index: usize) -> EntryPoint {
        self.action_entry_points[index]
    }

    pub unsafe fn from_raw_parts(
        id: u32,
        attrs: Vec<Word>,
        init_card_attrs: Vec<Word>,
        action_entry_points: Vec<EntryPoint>,
    ) -> Self {
        let mut card_type = CardType {
            id,
            attrs: Attrs::new(),
            init_card_attrs: TypeAttrs::new(),
            action_entry_points: EntryPoints::new(),
        };
        card_type.attrs.fill(attrs);
        card_type.init_card_attrs.fill(init_card_attrs);
        card_type.action_entry_points.fill(action_entry_points);
        card_type
    }
}

impl Default for CardType {
    fn default() -> Self {
        CardType {
            id: 0,
            attrs: TypeAttrs::new(),
            init_card_attrs: Attrs::new(),
            action_entry_points: EntryPoints::new(),
        }
    }
}
