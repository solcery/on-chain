use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum Event {
    BoardChange {
        attr_index: u32,
        previous_value: Word,
        new_value: Word,
    },
    CardChange {
        card_index: u32,
        attr_index: u32,
        previous_value: Word,
        new_value: Word,
    },
    AddCardById {
        card_index: u32,
        cargtype_id: u32,
    },
    AddCardByIndex {
        card_index: u32,
        cargtype_index: u32,
    },
    RemoveCard {
        card_id: u32,
    },
    CardActionStarted {
        cardtype_index: u32,
        action_index: u32,
        args: Vec<Word>,
    },
}

pub type Log = Vec<Event>;
