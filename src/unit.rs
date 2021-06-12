use borsh::{BorshDeserialize, BorshSerialize};
use std::marker::Copy;

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, Copy)]
pub struct Unit {
	pub id: u32,
	pub hp: u32,
	// pub team: usize,
}
