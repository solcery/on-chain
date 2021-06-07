use borsh::{BorshDeserialize, BorshSerialize};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Unit {
	pub id: u32,
	pub hp: u32,
}
