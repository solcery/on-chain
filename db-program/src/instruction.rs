use borsh::{BorshDeserialize, BorshSerialize};

pub use account_fs::SegmentId;
pub use solcery_db::{ColumnId, ColumnParams, Data, DataType};

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub enum DBInstruction {
    SetValue(SetValueParams),
    SetValueSecondary(SetValueSecondaryParams),
    SetRow(SetRowParams),
    DeleteRow(DeleteRowParams),
    DeleteRowSecondary(DeleteRowSecondaryParams),
    /// Create a new data base
    ///
    /// Accounts expected:
    ///
    /// 0. `[]` Global DB-program state account
    /// 1. `[signer]` Access Token account
    /// 2-... `[]` FS accounts
    CreateDB(CreateDBParams),
    DropDB(SegmentId),
    MintNewAccessToken,
    /// Bootstrap Solcery DB-program
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer, writable]` The account of the person, who will initiate DB.
    /// 1. `[writable]` Mint account
    /// 2. `[writable]` Global DB-program state account
    /// 3. `[signer,writable]` Access Token account
    /// 4. `[]` System Program
    /// 5. `[]` Token Program
    /// 6. `[]` Rent SysVar
    Bootstrap(BootstrapParams),
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub struct SetValueParams {
    pub db: SegmentId,
    pub column: ColumnId,
    pub key: Data,
    pub value: Data,
    /// Are all the FS accounts initialized
    pub is_initialized: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub struct SetValueSecondaryParams {
    pub db: SegmentId,
    pub key_column: ColumnId,
    pub secondary_key: Data,
    pub value_column: ColumnId,
    pub value: Data,
    /// Are all the FS accounts initialized
    pub is_initialized: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub struct SetRowParams {
    pub db: SegmentId,
    pub key: Data,
    pub row: Vec<(ColumnId, Data)>,
    /// Are all the FS accounts initialized
    pub is_initialized: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub struct DeleteRowParams {
    pub db: SegmentId,
    pub key: Data,
    /// Are all the FS accounts initialized
    pub is_initialized: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub struct DeleteRowSecondaryParams {
    pub db: SegmentId,
    pub secondary_key: Data,
    pub key_column: ColumnId,
    /// Are all the FS accounts initialized
    pub is_initialized: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub struct CreateDBParams {
    pub primary_key_type: DataType,
    pub columns: Vec<ColumnParams>,
    pub table_name: String,
    pub max_columns: u32,
    pub max_rows: u32,
    /// Are all the FS accounts initialized
    pub is_initialized: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub struct BootstrapParams {
    pub state_bump: u8,
    pub mint_bump: u8,
    pub lamports_to_global_state: u64,
    pub lamports_to_mint: u64,
}
