use borsh::{BorshDeserialize, BorshSerialize};
use crate::brick;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Unit {
	pub id: u32,
	pub hp: u32,
}

impl brick::ContextObject for Unit {
	fn damage(&mut self, amount: u32) -> () {
		self.hp -= amount;
	}
	fn heal(&mut self, amount: u32) -> () {
		self.hp += amount;
	}
}
