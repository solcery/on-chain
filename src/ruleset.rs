use borsh::{BorshDeserialize, BorshSerialize};




#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Ruleset {
	pub card_types: Vec<[u8; 32]>, //cards
	pub deck: Vec<(u32, Vec<(u32, u32)>)>, // place, card_id, amount
	pub client_data: Vec<u8>,
}