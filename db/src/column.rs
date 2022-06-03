use super::Data;
use borsh::{BorshDeserialize, BorshSerialize};
use slice_rbtree::RBTree;

pub trait Column {
    fn get_key(&self, value: Data) -> Option<Data>;
    fn get_value(&self, key: Data) -> Option<Data>;
    fn set(&mut self, key: Data, value: Data);
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

impl<'a, K, V, const KSIZE: usize, const VSIZE: usize> Column for RBTree<'a, K, V, KSIZE, VSIZE>
where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: BorshDeserialize + BorshSerialize,
{
    fn get_key(&self, value: Data) -> Option<Data> {
        unimplemented!();
    }
    fn get_value(&self, key: Data) -> Option<Data> {
        unimplemented!();
    }
    fn set(&mut self, key: Data, value: Data) {
        unimplemented!();
    }
}
