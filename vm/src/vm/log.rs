use crate::on_chain_types::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum Event {
    GameStateChange {
        attr_index: u32,
        previous_value: Word,
        new_value: Word,
    },
    ObjectChange {
        object_index: u32,
        attr_index: u32,
        previous_value: Word,
        new_value: Word,
    },
    AddObjectById {
        object_index: u32,
        object_type_id: u32,
    },
    AddObjectByIndex {
        object_index: u32,
        object_type_index: u32,
    },
    RemoveObject {
        object_id: u32,
    },
    ObjectActionStarted {
        object_type_index: u32,
        action_index: u32,
        args: Vec<Word>,
    },
}

pub type Log = Vec<Event>;
