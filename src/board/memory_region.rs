use crate::card::Card;
use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct MemoryRegion {
    pub cards: Vec<Card>,
    pub attrs: Vec<Word>,
}

impl MemoryRegion {
    pub fn new() -> Self {
        Self {
            cards: Vec::<Card>::new(),
            attrs: Vec::<Word>::new(),
        }
    }
    pub fn with_data(cards: Vec<Card>, attrs: Vec<Word>) -> Self {
        Self { cards, attrs }
    }
}
