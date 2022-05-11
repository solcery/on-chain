use super::Data;

pub trait Column {
    fn get_key(&self, value: Data) -> Option<Data>;
    fn get_value(&self, key: Data) -> Option<Data>;
    fn set(&mut self, key: Data, value: Data);
    fn keys(&self) -> KeysAscendingIterator;
    fn values(&self) -> ValuesAscendingIterator;
    fn pairs(&self) -> PairsAscendingIterator;
    fn keys_descending(&self) -> KeysDescendingIterator;
    fn values_descending(&self) -> ValuesDescendingIterator;
    fn pairs_descending(&self) -> PairsDescendingIterator;
}

pub struct KeysAscendingIterator {}
pub struct ValuesAscendingIterator {}
pub struct PairsAscendingIterator {}
pub struct KeysDescendingIterator {}
pub struct ValuesDescendingIterator {}
pub struct PairsDescendingIterator {}
