use borsh::{BorshDeserialize, BorshSerialize};

use super::word::Word;

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Object {
    id: u32,
    object_type: u32,
    // На данный момент мы считаем все атрибуты статическими в том смысле, что у всех карт attrs[i]
    // имеет один и тот же игровой смысл.
    // Если мы оставляем это так, то имеет смысл сделать attrs приватным полем и сделать на него
    // сеттер таким образом, чтобы он не мог поменять num на bool
    pub attrs: Vec<Word>,
}

impl Object {
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[must_use]
    pub fn object_type(&self) -> u32 {
        self.object_type
    }

    #[must_use]
    pub unsafe fn from_raw_parts(id: u32, object_type: u32, attrs: Vec<Word>) -> Self {
        Self {
            id,
            object_type,
            attrs,
        }
    }
}
