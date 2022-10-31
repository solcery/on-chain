use borsh::{BorshDeserialize, BorshSerialize};

use super::object::Object;
use super::word::Word;

pub const BOARD_ACCOUNT_SIZE: usize = 1024;

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct GameState {
    pub objects: Vec<Object>,
    pub attrs: Vec<Word>,
    object_index: u32,
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}

impl GameState {
    #[must_use]
    pub fn new() -> Self {
        Self {
            objects: Vec::<Object>::new(),
            attrs: Vec::<Word>::new(),
            object_index: 0,
        }
    }

    pub fn generate_object_id(&mut self) -> u32 {
        let id = self.object_index;
        self.object_index += 1;
        id
    }

    #[must_use]
    pub unsafe fn from_raw_parts(
        objects: Vec<Object>,
        attrs: Vec<Word>,
        object_index: u32,
    ) -> Self {
        Self {
            objects,
            attrs,
            object_index,
        }
    }
}
