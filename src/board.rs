use crate::card::Card;
use crate::word::Word;
use serde::{Deserialize, Serialize};
use tinyvec::ArrayVec;

const DECK_SIZE: usize = 2_usize.pow(7);
type Deck = ArrayVec<[Card; DECK_SIZE]>;

const ATTR_VEC_SIZE: usize = 2_usize.pow(5);
type AttrVec = ArrayVec<[Word; ATTR_VEC_SIZE]>;

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

    pub unsafe fn from_raw_parts(cards: Vec<Card>, attrs: Vec<Word>, card_index: u32) -> Board {
        let mut board = Board {
            cards: Deck::new(),
            attrs: AttrVec::new(),
            card_index,
        };
        board.cards.fill(cards);
        board.attrs.fill(attrs);
        board
    }
}
