use borsh::{BorshDeserialize, BorshSerialize};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};

use super::DataType;

#[derive(PartialEq, Clone, Eq, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ColumnParams {
    name: String,
    data_type: DataType,
    column_type: ColumnType,
}

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
    RBTree,
    OneToOne,
    // This types are not implemented yet
    //OneToMany,
    //ManyToOne,
    //ManyToMany,
    //RBSet,
}
