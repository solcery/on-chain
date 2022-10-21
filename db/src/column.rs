use std::fmt::Debug;

use super::Data;
use super::Error;

/// An interface for operations with columns
///
/// Any data structure can be used as a column in the [`DB`](crate::DB), as long as it implements
/// this trait
pub trait Column: Debug {
    fn get_key(&self, value: Data) -> Option<Data>;
    fn get_value(&self, key: Data) -> Option<Data>;
    fn set(&mut self, key: Data, value: Data) -> Result<Option<Data>, Error>;
    fn delete_by_key(&mut self, key: Data) -> bool;
    fn delete_by_value(&mut self, value: Data) -> bool;
    //fn keys(&self) -> KeysAscendingIterator;
    //fn values(&self) -> ValuesAscendingIterator;
    //fn pairs(&self) -> PairsAscendingIterator;
    //fn keys_descending(&self) -> KeysDescendingIterator;
    //fn values_descending(&self) -> ValuesDescendingIterator;
    //fn pairs_descending(&self) -> PairsDescendingIterator;
}

//pub struct KeysAscendingIterator {}
//pub struct ValuesAscendingIterator {}
//pub struct PairsAscendingIterator {}
//pub struct KeysDescendingIterator {}
//pub struct ValuesDescendingIterator {}
//pub struct PairsDescendingIterator {}
