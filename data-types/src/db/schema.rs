use borsh::{BorshDeserialize, BorshSerialize};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

use super::column::Column as ColumnTrait;
use super::error::Error;
use solcery_impl_generator::generate_column_impls;

#[generate_column_impls(
    Data,
    ColumnTrait,
    Error,
    derives(
        PartialEq,
        Clone,
        Eq,
        Debug,
        BorshSerialize,
        BorshDeserialize,
        Serialize,
        Deserialize
    )
)]
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
pub enum DataType {
    #[type_params(i32, 4)]
    Int,
    #[type_params(Pubkey, 64)]
    Pubkey,
    #[type_params(String, 16)]
    ShortString,
    #[type_params(String, 64)]
    MediumString,
    #[type_params(String, 256)]
    LongString,
}

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
