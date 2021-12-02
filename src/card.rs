use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Card {
    id: u32,
    card_type: u32,
    // На данный момент мы считаем все атрибуты статическими в том смысле, что у всех карт attrs[i]
    // имеет один и тот же игровой смысл.
    // Если мы оставляем это так, то имеет смысл сделать attrs приватным полем и сделать на него
    // сеттер таким образом, чтобы он не мог поменять num на bool
    pub attrs: Vec<Word>,
}

impl Card {
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[must_use]
    pub fn card_type(&self) -> u32 {
        self.card_type
    }

    #[must_use]
    pub unsafe fn from_raw_parts(id: u32, card_type: u32, attrs: Vec<Word>) -> Self {
        Self {
            id,
            card_type,
            attrs,
        }
    }
}

impl Default for Card {
    fn default() -> Self {
        Self {
            id: 0,
            card_type: 0,
            attrs: Vec::<Word>::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct EntryPoint {
    address: u32,
    n_args: u32,
}

impl EntryPoint {
    #[must_use]
    pub fn address(&self) -> usize {
        self.address as usize
    }

    #[must_use]
    pub fn n_args(&self) -> usize {
        self.n_args as usize
    }

    #[must_use]
    pub unsafe fn from_raw_parts(address: u32, n_args: u32) -> Self {
        Self { address, n_args }
    }
}

impl Default for EntryPoint {
    fn default() -> Self {
        Self {
            address: 0,
            n_args: 0,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct CardType {
    id: u32,
    attrs: Vec<Word>,
    init_card_attrs: Vec<Word>,
    action_entry_points: Vec<EntryPoint>,
}

impl CardType {
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[must_use]
    pub fn attr_by_index(&self, index: usize) -> Word {
        self.attrs[index]
    }

    #[must_use]
    pub fn instantiate_card(&self, id: u32) -> Card {
        Card {
            id,
            card_type: self.id(),
            attrs: self.init_card_attrs.clone(),
        }
    }

    #[must_use]
    pub fn action_entry_point(&self, index: usize) -> EntryPoint {
        self.action_entry_points[index]
    }

    #[must_use]
    pub unsafe fn from_raw_parts(
        id: u32,
        attrs: Vec<Word>,
        init_card_attrs: Vec<Word>,
        action_entry_points: Vec<EntryPoint>,
    ) -> Self {
        Self {
            id,
            attrs,
            init_card_attrs,
            action_entry_points,
        }
    }
}

impl Default for CardType {
    fn default() -> Self {
        Self {
            id: 0,
            attrs: Vec::<Word>::new(),
            init_card_attrs: Vec::<Word>::new(),
            action_entry_points: Vec::<EntryPoint>::new(),
        }
    }
}
