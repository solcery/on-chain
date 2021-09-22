use crate::card::{Card, CardType};
use crate::word::Word;
use std::convert::TryInto;
use tinyvec::ArrayVec;

const DECK_SIZE: usize = 512;
type Deck = ArrayVec<[Card; DECK_SIZE]>;

const TYPEDECK_SIZE: usize = 256;
type TypeDeck = ArrayVec<[CardType; TYPEDECK_SIZE]>;

const ATTR_VEC_SIZE: usize = 128;
type AttrVec = ArrayVec<[Word; ATTR_VEC_SIZE]>;

#[derive(Debug)]
pub struct Board {
    card_types: TypeDeck,
    pub cards: Deck,
    pub attrs: AttrVec,
}

impl Board {
    pub fn new() -> Self {
        Board {
            card_types: TypeDeck::new(),
            cards: Deck::new(),
            attrs: AttrVec::new(),
        }
    }

    pub fn add_type(&mut self, typ: CardType) {
        self.card_types.push(typ);
    }

    pub fn card_type_count(&self) -> usize {
        self.card_types.len()
    }

    pub fn card_type_by_type_index(&self, type_index: usize) -> &CardType {
        &self.card_types[type_index]
    }

    pub fn card_type_by_type_id(&self, type_id: u32) -> Option<&CardType> {
        self.card_types
            .iter()
            .find(|card_type| card_type.id() == type_id)
    }

    pub fn instance_card_by_type_id(&mut self, type_id: u32, card_id: u32) -> Result<(), ()> {
        let typ = &self.card_types.iter().find(|typ| typ.id() == type_id);
        match typ {
            Some(typ) => {
                self.cards.push(typ.instantiate_card(card_id));
                Ok(())
            }
            None => Err(()),
        }
    }

    pub fn instance_card_by_type_index(&mut self, type_index: u32, card_id: u32) -> Result<(), ()> {
        let index = type_index.try_into().unwrap();
        if self.card_types.len() < index {
            let typ = &self.card_types[index];
            self.cards.push(typ.instantiate_card(card_id));
            Ok(())
        } else {
            Err(())
        }
    }

    #[cfg(test)]
    pub fn prepare_board(card_types: Vec<CardType>, cards: Vec<Card>, attrs: Vec<Word>) -> Board {
        let mut board = Board {
            card_types: TypeDeck::new(),
            cards: Deck::new(),
            attrs: AttrVec::new(),
        };
        board.card_types.fill(card_types);
        board.cards.fill(cards);
        board.attrs.fill(attrs);
        board
    }
}
