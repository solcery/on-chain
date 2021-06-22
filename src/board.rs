use borsh::{BorshDeserialize, BorshSerialize};
use std::marker::Copy;
use solana_program::{
    pubkey::Pubkey,
    msg,
};
use crate::unit::Unit;
use crate::brick::BorshResult;
use std::cell::RefCell;
use std::io::Write;
use std::rc::{Rc, Weak};

pub type PlaceId = (u32, u32);


pub struct Place {
	pub id: PlaceId,
	pub unit: Option<Weak<RefCell<Unit>>>,
}

pub struct Board {
	pub units: Vec<Rc<RefCell<Unit>>>, //4 + len * Unit.size. Max size is 8
}

impl BorshSerialize for Board {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		(self.units.len() as u32).serialize(writer);
		for unit in self.units.iter() {
			unit.borrow().serialize(writer);
		}
		Ok(())
	}
}

impl BorshDeserialize for Board {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> {
		let units_len = u32::deserialize(buf)?;
		let mut units = Vec::new();
		for i in 0..units_len {
			let unit = Unit::deserialize(buf)?;
			units.push(Rc::new(RefCell::new(unit)));
		}
		Ok(Board {
			units,
		})
	}
}

impl Board{
	pub fn new(size_x: u32, size_y: u32) -> Board {
		let mut places = Vec::new();
		for x in 0..size_x {
			for y in 0..size_y {
				places.push( Place {
					id: (x, y),
					unit: None,
				});
			}
		}
		return Board { 
			units: Vec::new(),
		}
	}

	pub fn get_unit_by_place(&self, place_id: PlaceId) -> Option<Rc<RefCell<Unit>>> {
		for unit in self.units.iter() {
			if unit.borrow().place == place_id {
				return Some(Rc::clone(&unit))
			}
		}
		return None
	}

	pub fn get_unit_by_type(&self, unit_type: Pubkey) -> Option<Rc<RefCell<Unit>>> {
		for unit in self.units.iter() {
			if unit.borrow().unit_type == unit_type {
				return Some(Rc::clone(&unit))
			}
		}
		return None
	}

	pub fn get_unit_by_id(&self, id: u32) -> Option<Rc<RefCell<Unit>>> {
		return Some(Rc::clone(&self.units[id as usize]))
	}

	pub fn create_unit(&mut self, owner:Pubkey, unit_type: Pubkey, place: PlaceId) {
		let new_unit = Unit {
			owner,
			unit_type,
			place,
			hp: 20,
		};
		self.units.push(Rc::new(RefCell::new(new_unit)));
	}
}


