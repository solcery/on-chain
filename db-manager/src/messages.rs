use borsh::{BorshDeserialize, BorshSerialize};
use solcery_data_types::db::schema_id::SchemaId;

use crate::db_manager::{DBId, DBQuery};

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
