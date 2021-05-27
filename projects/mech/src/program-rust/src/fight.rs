use borsh::{BorshDeserialize, BorshSerialize};
use crate::unit::Unit;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Fight {
	//id: u32,
	units: Vec<Unit>,	
}

impl Fight {
	pub fn new() -> Fight {
		let mut units = Vec::new();
		let first_unit = Unit {
			id: 1,
			hp: 20,
		};
		let second_unit = Unit {
			id: 1,
			hp: 20,
		};
		units.push(first_unit);
		units.push(second_unit);
		return Fight { units }
	}
	
	pub fn make_move(unit_id: u32, target_id: u32) {
	
	}
	
	pub fn dump_fight() {
	
	}
}

