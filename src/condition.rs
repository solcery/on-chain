use crate::brick::{ Context, Brick, BorshResult, Condition, Value };
use std::io::Write;
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryInto;
use crate::board::Place;

impl BorshSerialize for Condition {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		let condition_code = 1u32.to_le_bytes();
		let code = self.get_code();
		writer.write_all(&condition_code)?;
		writer.write_all(&code.to_le_bytes())?;
		let x = self.b_to_vec();
		writer.write_all(&x)?;
		Ok(())
	}
}

impl BorshDeserialize for Condition {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> { 
		let _condition_code = u32::from_le_bytes(buf[..4].try_into().unwrap());
		*buf = &buf[4..];
		let code = u32::from_le_bytes(buf[..4].try_into().unwrap());
		*buf = &buf[4..];
		match code {
			0u32 => Ok(Box::new(True::deserialize(buf)?)),
			1u32 => Ok(Box::new(False::deserialize(buf)?)),
			2u32 => Ok(Box::new(Or::deserialize(buf)?)),
			3u32 => Ok(Box::new(And::deserialize(buf)?)),
			4u32 => Ok(Box::new(Not::deserialize(buf)?)),
			5u32 => Ok(Box::new(Equal::deserialize(buf)?)),
			6u32 => Ok(Box::new(GreaterThan::deserialize(buf)?)),
			7u32 => Ok(Box::new(LesserThan::deserialize(buf)?)),
			100u32 => Ok(Box::new(IsAtPlace::deserialize(buf)?)),
			_ => Ok(Box::new(True::deserialize(buf)?)),
		}
	}
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct True {}

impl Brick<bool> for True {
	fn get_code(&self) -> u32 {
		return 0u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, _ctx: &mut Context) -> bool {	
		return true
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct False {}

impl Brick<bool> for False {
	fn get_code(&self) -> u32 {
		return 1u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, _ctx: &mut Context) -> bool {	
		return false
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Or {
	pub condition1: Condition,
	pub condition2: Condition,
}

impl Brick<bool> for Or {
	fn get_code(&self) -> u32 {
		return 2u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> bool {	
		self.condition1.run(ctx) || self.condition2.run(ctx)
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct And {
	pub condition1: Condition,
	pub condition2: Condition,
}

impl Brick<bool> for And {
	fn get_code(&self) -> u32 {
		return 3u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> bool {	
		self.condition1.run(ctx) && self.condition2.run(ctx)
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Not {
	pub condition: Condition,
}

impl Brick<bool> for Not {
	fn get_code(&self) -> u32 {
		return 4u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> bool {	
		!self.condition.run(ctx)
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Equal {
	pub value1: Value,
	pub value2: Value,
}

impl Brick<bool> for Equal {
	fn get_code(&self) -> u32 {
		return 5u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> bool {	
		return self.value1.run(ctx) == self.value2.run(ctx);
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GreaterThan {
	pub value1: Value,
	pub value2: Value,
}

impl Brick<bool> for GreaterThan {
	fn get_code(&self) -> u32 {
		return 6u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> bool {	
		return self.value1.run(ctx) > self.value2.run(ctx)
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct LesserThan {
	pub value1: Value,
	pub value2: Value,
}

impl Brick<bool> for LesserThan {
	fn get_code(&self) -> u32 {
		return 7u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> bool {	
		return self.value1.run(ctx) < self.value2.run(ctx);
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct IsAtPlace {
	pub place: Value,
}

impl Brick<bool> for IsAtPlace {
	fn get_code(&self) -> u32 {
		return 100u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> bool {	
		let place = self.place.run(ctx);
		return ctx.object.borrow().place == Place::from_i32(place);
	}	
}
