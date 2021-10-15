use crate::card::Card;
use crate::word::Word;
use serde::{Deserialize, Serialize};
use tinyvec::ArrayVec;

const DECK_SIZE: usize = 128;
type Deck = Vec<Card>;

const ATTR_VEC_SIZE: usize = 64;
type AttrVec = Vec<Word>;

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Board {
    pub cards: Deck,
    pub attrs: AttrVec,
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
            cards: Deck::new(),
            attrs: AttrVec::new(),
            card_index: 0,
        }
    }

    pub fn generate_card_id(&mut self) -> u32 {
        let id = self.card_index;
        self.card_index += 1;
        id
    }

    #[cfg(test)]
    pub unsafe fn from_raw_parts(cards: Vec<Card>, attrs: Vec<Word>, card_index: u32) -> Board {
        Board {
            cards,
            attrs,
            card_index,
        }
    }
}
