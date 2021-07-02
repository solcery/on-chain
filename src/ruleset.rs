use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
	account_info::AccountInfo,
    pubkey::Pubkey,
    msg,
};
use crate::brick::{
	BorshResult, 
	Action
};
use std::io::Write;

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub struct Ruleset {
	pub card_types: Vec<[u8; 32]>, //cards
	pub deck: Vec<(u32, u32, u32)>, // place, card_id, amount
	pub initializers: Vec<u32>,
	pub client_data: Vec<u8>,
}