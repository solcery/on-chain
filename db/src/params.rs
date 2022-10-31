use borsh::{BorshDeserialize, BorshSerialize};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

use super::DataType;

/// Parameters of the column to be inserted in the [`DB`](crate::DB)
#[derive(PartialEq, Clone, Eq, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ColumnParams {
    name: String,
    data_type: DataType,
    column_type: ColumnType,
}

/// Possible containers for the column data
#[derive(
    PartialEq,
    Copy,
    Clone,
    Eq,
    Debug,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[repr(u8)]
pub enum ColumnType {
    /// if data is stored in the [`RBTree`](slice_rbtree::tree::RBTree), it can not be used as a
    /// secondary  key
    RBTree,
    /// if key-value pairs is stored in the [`OneToOne`](solcery_reltab::one_to_one::OneToOne)
    /// container, they can be fetched by both key and value
    OneToOne,
    // This types are not implemented yet
    //OneToMany,
    //ManyToOne,
    //ManyToMany,
    //RBSet,
}
