use crate::card::Card;
use crate::word::Word;
use tinyvec::ArrayVec;

const DECK_SIZE: usize = 512;
type Deck = ArrayVec<[Card; DECK_SIZE]>;

const ATTR_VEC_SIZE: usize = 128;
type AttrVec = ArrayVec<[Word; ATTR_VEC_SIZE]>;

#[derive(Debug, Clone)]
pub struct Board {
    pub cards: Deck,
    pub attrs: AttrVec,
}

impl Board {
    pub fn new() -> Self {
        Board {
            cards: Deck::new(),
            attrs: AttrVec::new(),
        }
    }

    #[cfg(test)]
    pub unsafe fn prepare_board(cards: Vec<Card>, attrs: Vec<Word>) -> Board {
        let mut board = Board {
            cards: Deck::new(),
            attrs: AttrVec::new(),
        };
        board.cards.fill(cards);
        board.attrs.fill(attrs);
        board
    }
}
