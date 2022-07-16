use slice_rbtree::{Error, RBTree};
use solcery_impl_generator::generate_column_impls;

pub trait Column {
    fn get_key(&self, value: HolderName) -> Option<HolderName>;
    fn get_value(&self, key: HolderName) -> Option<HolderName>;
    fn set(&mut self, key: HolderName, value: HolderName) -> Option<HolderName>;
    fn delete_by_key(&mut self, key: HolderName) -> bool;
    fn delete_by_value(&mut self, value: HolderName) -> bool;
}

pub enum ErrorType {
    Test,
}

pub enum ColumnType {
    RBTree,
}

impl From<Error> for ErrorType {
    fn from(_err: Error) -> Self {
        Self::Test
    }
}

#[generate_column_impls(HolderName, Column, ErrorType, derives(Debug))]
pub enum Test {
    #[type_params(i32, 4)]
    Int,
    #[type_params(u64, 8)]
    Unsigned,
}
