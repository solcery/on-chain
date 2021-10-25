use crate::card::Card;
use crate::word::Word;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
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
    pub fn new() -> Self {
        Board {
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

    pub unsafe fn from_raw_parts(cards: Vec<Card>, attrs: Vec<Word>, card_index: u32) -> Board {
        Board {
            cards,
            attrs,
            card_index,
        }
    }
}
