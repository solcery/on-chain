use crate::card::Card;
use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct MemoryRegion {
    owner_id: u32,
    pub cards: Vec<Card>,
    pub attrs: Vec<Word>,
}

impl MemoryRegion {
    pub fn new(owner_id: u32) -> Self {
        Self {
            owner_id,
            cards: Vec::<Card>::new(),
            attrs: Vec::<Word>::new(),
        }
    }
    pub fn with_data(owner_id: u32, cards: Vec<Card>, attrs: Vec<Word>) -> Self {
        Self {
            owner_id,
            cards,
            attrs,
        }
    }

    #[must_use]
    pub fn owner(&self) -> u32 {
        self.owner_id
    }
}
