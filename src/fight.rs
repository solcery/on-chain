use borsh::{BorshDeserialize, BorshSerialize};
use crate::unit::Unit;
use solana_program::{
    pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Fight {
	pub units: Vec<Unit>,
}


impl Fight {
	pub fn new(_owner: Pubkey) -> Fight {
		let mut units = Vec::new();
		let first_unit = Unit {
			id: 1,
			hp: 20,
		};
		let second_unit = Unit {
			id: 2,
			hp: 20,
		};
		units.push(first_unit);
		units.push(second_unit);

		return Fight { units }
	}
}
