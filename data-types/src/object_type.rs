use crate::object::Object;
use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct ObjectType {
    id: u32,
    attrs: Vec<Word>,
    init_object_attrs: Vec<Word>,
    action_entry_points: Vec<EntryPoint>,
}

impl ObjectType {
    #[must_use]
    pub fn id(&self) -> u32 {
        self.id
    }

    #[must_use]
    pub fn attr_by_index(&self, index: usize) -> Word {
        self.attrs[index]
    }

    #[must_use]
    pub fn instantiate_object(&self, id: u32) -> Object {
        unsafe { Object::from_raw_parts(id, self.id(), self.init_object_attrs.clone()) }
    }

    #[must_use]
    pub fn action_entry_point(&self, index: usize) -> EntryPoint {
        self.action_entry_points[index]
    }

    #[must_use]
    pub unsafe fn from_raw_parts(
        id: u32,
        attrs: Vec<Word>,
        init_object_attrs: Vec<Word>,
        action_entry_points: Vec<EntryPoint>,
    ) -> Self {
        Self {
            id,
            attrs,
            init_object_attrs,
            action_entry_points,
        }
    }
}

impl Default for ObjectType {
    fn default() -> Self {
        Self {
            id: 0,
            attrs: Vec::<Word>::new(),
            init_object_attrs: Vec::<Word>::new(),
            action_entry_points: Vec::<EntryPoint>::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct EntryPoint {
    address: u32,
    n_args: u32,
}

impl EntryPoint {
    #[must_use]
    pub fn address(&self) -> usize {
        self.address as usize
    }

    #[must_use]
    pub fn n_args(&self) -> usize {
        self.n_args as usize
    }

    #[must_use]
    pub unsafe fn from_raw_parts(address: u32, n_args: u32) -> Self {
        Self { address, n_args }
    }
}

impl Default for EntryPoint {
    fn default() -> Self {
        Self {
            address: 0,
            n_args: 0,
        }
    }
}
