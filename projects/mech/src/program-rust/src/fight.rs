use borsh::{BorshDeserialize, BorshSerialize};
use crate::brick::{ BorshResult };
use std::io::Write;
use crate::unit::Unit;
use solana_program::{
    pubkey::Pubkey,
};
use std::convert::TryInto;

pub struct Fight {
	pub owner: Pubkey,
	pub units: Vec<Unit>,
}

impl BorshSerialize for Fight {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		writer.write_all(&self.owner.to_bytes());
		self.units.serialize(writer);
		Ok(())
	}
}

impl BorshDeserialize for Fight {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> { 
		let owner = Pubkey::new(buf[..32].try_into().unwrap());
		let units = Vec::<Unit>::try_from_slice(&buf[33..]).unwrap();
		Ok(Fight { owner, units })
	}
}


impl Fight {
	pub fn new(owner: Pubkey) -> Fight {
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

		return Fight { owner, units }
	}
	// pub fn get_units(&mut self) -> Vec<&mut Unit> {
	// 	let x: Vec<&mut Unit> = Vec::new();
	// 	for i in 0..self.units.len() {
	// 		x.push(&mut self.units[i]);
	// 	}
	// 	return x;
	// }
}

