use crate::db::{
    schema::{AllowedTypes, Schema},
    schema_id::SchemaId,
};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct AddSchema {
    pub id: SchemaId,
    pub schema: Schema,
    pub need_init: bool,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct RemoveSchema {
    pub id: SchemaId,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct UpdateSchema {
    pub id: SchemaId,
    pub tables: Vec<AllowedTypes>,
}
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct GetSchema {
    pub id: SchemaId,
}
