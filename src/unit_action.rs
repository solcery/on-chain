use crate::brick::{ 
	Context, Brick, BorshResult, UnitAction
};
use std::io::Write;
use borsh::{
	BorshDeserialize, BorshSerialize
};
use std::convert::TryInto;
use std::cmp;

impl BorshSerialize for UnitAction {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		let action_code = 0u32.to_le_bytes();
		let code = self.get_code();
		writer.write_all(&action_code)?;
		writer.write_all(&code.to_le_bytes())?;
		let x = self.b_to_vec();
		writer.write_all(&x)?;
		Ok(())
	}
}

impl BorshDeserialize for UnitAction {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> { 
		let _action_code = u32::from_le_bytes(buf[..4].try_into().unwrap());
		*buf = &buf[4..];
		let code = u32::from_le_bytes(buf[..4].try_into().unwrap());
		*buf = &buf[4..];
		match code {
			0u32 => Ok(Box::new(MoveTo::deserialize(buf)?)),
			1u32 => Ok(Box::new(Attack::deserialize(buf)?)),
			_ => Ok(Box::new(MoveTo::deserialize(buf)?)),
		}
	}
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MoveTo {
	pub object_index: u32,
}

impl Brick<()> for MoveTo {
	fn get_code(&self) -> u32 {
		return 0u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () {
		let mut obj = ctx.objects[self.object_index as usize].borrow_mut();
		let unit_on_place = ctx.board.get_unit_by_place(ctx.place);
		if unit_on_place.is_some() {
			return;
		}
		let (x, y) = obj.place;
		let (new_x, new_y) = ctx.place;
		let distance: i32 = (new_x as i32 + new_y as i32) - (x as i32 + y as i32);
		if distance.abs() <= 1 {
			obj.place = ctx.place;
		}
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Attack {
	pub dmg: u32,
}

impl Brick<()> for Attack {
	fn get_code(&self) -> u32 {
		return 1u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () { // TOO MUCH
		let caster = ctx.objects[0].borrow_mut(); // caster should exist and be alive
		if caster.hp == 0 {
			return
		}
		let dmg = self.dmg;
		let obj_rc = match ctx.board.get_unit_by_place(ctx.place) {
			Some(obj_rc) => obj_rc,
			None => return,
		};
		let mut obj = obj_rc.borrow_mut();
		obj.hp = cmp::max(obj.hp - dmg, 0);
	}	
}