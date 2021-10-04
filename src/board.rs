use crate::card::Card;
use crate::word::Word;
use tinyvec::SliceVec;

const DECK_SIZE: usize = 512;
type Deck<'a> = SliceVec<'a, Card>;

const ATTR_VEC_SIZE: usize = 128;
type AttrVec<'a> = SliceVec<'a, Word>;

#[derive(Debug, Eq, PartialEq)]
pub struct Board<'a> {
    pub cards: Deck<'a>,
    pub attrs: AttrVec<'a>,
    card_index: u32,
}

impl<'a, 'b> Board<'a> {
    pub fn new_from_slices(deck_slice: &'a mut [Card], attr_slice: &'a mut [Word]) -> Self {
        Board {
            cards: Deck::from_slice_len(deck_slice, 0),
            attrs: AttrVec::from_slice_len(attr_slice, 0),
            card_index: 0,
        }
    }

    pub fn clone_to_slices(&self, deck_slice: &'b mut [Card], attr_slice: &'b mut [Word]) -> Board<'b> {
        assert!(deck_slice.len() >= self.cards.len());
        assert!(attr_slice.len() >= self.attrs.len());

        let mut board = Board {
            cards: Deck::from_slice_len(deck_slice, 0),
            attrs: AttrVec::from_slice_len(attr_slice, 0),
            card_index: self.card_index,
        };
        board.cards.extend_from_slice(&self.cards);
        board.attrs.extend_from_slice(&self.attrs);
        board
    }

    pub fn generate_card_id(&mut self) -> u32 {
        let id = self.card_index;
        self.card_index += 1;
        id
    }

    #[cfg(test)]
    pub unsafe fn from_raw_parts(
        deck_slice: &'a mut [Card],
        attr_slice: &'a mut [Word],
        cards: &'a [Card],
        attrs: &'a [Word],
        card_index: u32,
    ) -> Self {
        let mut board = Board {
            cards: Deck::from_slice_len(deck_slice, 0),
            attrs: AttrVec::from_slice_len(attr_slice, 0),
            card_index,
        };
        board.cards.extend_from_slice(cards);
        board.attrs.extend_from_slice(attrs);
        board
    }
}
