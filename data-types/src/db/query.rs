use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use Vec;

use super::messages::db_manager::DBId;

#[derive(PartialEq, Eq, Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ReadQuery<K> {
    pub db_id: DBId,
    pub keys: Vec<K>,
}

#[derive(PartialEq, Eq, Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct WriteQuery<K, V> {
    pub db_id: DBId,
    pub pairs: Vec<(K, V)>,
}

#[derive(PartialEq, Eq, Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct RemoveQuery<K> {
    pub db_id: DBId,
    pub keys: Vec<K>,
}
