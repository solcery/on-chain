use crate::db::schema_id::SchemaId;
use borsh::{BorshDeserialize, BorshSerialize};

// TODO: specify by DB
pub type DBId = String;
pub type DBQuery = String;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct CreateDB {
    schema_id: SchemaId,
    db_id: DBId,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct RemoveDB {
    db_id: DBId,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Query {
    message: DBId,
    query: DBQuery,
}
