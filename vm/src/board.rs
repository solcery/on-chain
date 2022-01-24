use crate::card::Card;
use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};

pub const BOARD_ACCOUNT_SIZE: usize = 1024;

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Board {
    pub cards: Vec<Card>,
    pub attrs: Vec<Word>,
    card_index: u32,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cards: Vec::<Card>::new(),
            attrs: Vec::<Word>::new(),
            card_index: 0,
        }
    }

    pub fn generate_card_id(&mut self) -> u32 {
        let id = self.card_index;
        self.card_index += 1;
        id
    }

    #[must_use]
    pub unsafe fn from_raw_parts(cards: Vec<Card>, attrs: Vec<Word>, card_index: u32) -> Self {
        Self {
            cards,
            attrs,
            card_index,
        }
    }
}
