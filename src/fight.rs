use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Fight {
	pub log: Vec<LogEntry>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct LogEntry {
	pub player_id: u32,
	pub card_id: u32,
}
