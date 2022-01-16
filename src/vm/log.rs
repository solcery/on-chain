use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum Event {
    RegionChange {
        region_index: u16,
        attr_index: u16,
        previous_value: Word,
        new_value: Word,
    },
    CardChange {
        region_index: u16,
        card_index: u32,
        attr_index: u16,
        previous_value: Word,
        new_value: Word,
    },
    AddCardById {
        card_index: u32,
        cardtype_id: u32,
    },
    AddCardByIndex {
        card_index: u32,
        cardtype_index: u32,
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
