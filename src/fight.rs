use borsh::{BorshDeserialize, BorshSerialize};
use crate::unit::Unit;
use solana_program::{
    pubkey::Pubkey,
};
use std::collections::HashMap;
use std::cell::RefCell;

#[derive(BorshSerialize, BorshDeserialize)]
#[borsh_init(init)]
pub struct Fight {
	pub owner: [u8; 32],
	pub uniq_id_seq: u32,
	pub attrs: HashMap<u32, u32>,
	#[borsh_skip]
	pub units: HashMap<u32, RefCell<Unit>>,
	pub units_serialized: Vec<Unit>,
}

impl Fight {
    pub fn init(&mut self) {
    	// let unit: &mut Vec<Unit> = &mut self.units_serialized;
    	// let rest: &mut Vec<Unit>;
    	for unit in self.units_serialized.iter() {
    		// let(unit, rest) = unit.split_at_mut(1);
    		self.units.insert(unit.id, RefCell::new(*unit));
    	}
    }
}


impl Fight {
	pub fn new(owner: Pubkey) -> Fight {
		let mut units = Vec::new();
		let first_unit = Unit {
			id: 0,
			hp: 20,
		};
		let second_unit = Unit {
			id: 1,
			hp: 20,
		};
		units.push(first_unit);
		units.push(second_unit);

		return Fight { 
			//units: HashMap::new(),
			units_serialized: Vec::new(),
			units: HashMap::new(), 
			attrs: HashMap::new(),
			uniq_id_seq: 0u32,
			owner: owner.to_bytes(),
		}
	}

	pub fn create_unit(&mut self, _unit_type: Pubkey, team: usize) {
		self.uniq_id_seq += 1;
		let new_unit = Unit {
			hp: 20,
			id: self.uniq_id_seq,
		};
		self.units.insert(new_unit.id, RefCell::new(new_unit));

	}
}
