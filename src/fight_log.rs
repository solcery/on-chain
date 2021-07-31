use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct FightLog {
	pub log: Vec<LogEntry>,
}

#[derive(Debug, BorshDeserialize, BorshSerialize, Clone)]
pub struct LogEntry {
	pub player_id: u32,
	pub action_type: u32,
	pub action_data: u32,
}
