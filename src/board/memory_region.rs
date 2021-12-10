use crate::card::Card;
use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
struct MemoryRegion {
    owner_id: u32,
    cards: Vec<Card>,
    attrs: Vec<Word>,
}
