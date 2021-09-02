use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Player {
	pub id: Pubkey,
	pub bool_attrs: Vec<bool>,
	pub numeral_attrs: Vec<i32>,
}
