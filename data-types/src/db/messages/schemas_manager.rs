use crate::db::{schema::Schema, schema_id::SchemaId};
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct AddSchema {
    pub id: SchemaId,
    pub schema: Schema,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct RemoveSchema {
    pub id: SchemaId,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct UpdateSchema {
    pub id: SchemaId,
    pub new_schema: Schema,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct GetSchema {
    pub id: SchemaId,
}
