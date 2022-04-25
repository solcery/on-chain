use crate::schemas_manager::Schema;
use borsh::{BorshDeserialize, BorshSerialize};
use solcery_data_types::db::schema_id::SchemaId;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct AddSchema {
    id: SchemaId,
    schema: Schema,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct RemoveSchema {
    id: SchemaId,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct UpdateSchema {
    id: SchemaId,
    new_schema: Schema,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct GetSchema {
    id: SchemaId,
}
