use crate::brick::{ Context, Brick, BorshResult, Value, Condition};
use std::io::Write;
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryInto;

impl BorshSerialize for Value {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		let value_code = 2u32.to_le_bytes();
		let code = self.get_code();
		writer.write_all(&value_code)?;
		writer.write_all(&code.to_le_bytes())?;
		let x = self.b_to_vec();
		writer.write_all(&x)?;
		Ok(())
	}
}

impl BorshDeserialize for Value {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> { 
		let _value_code = u32::from_le_bytes(buf[..4].try_into().unwrap());
		*buf = &buf[4..];
		let code = u32::from_le_bytes(buf[..4].try_into().unwrap());
		*buf = &buf[4..];
		match code {
			0u32 => Ok(Box::new(Const::deserialize(buf)?)),
			1u32 => Ok(Box::new(Conditional::deserialize(buf)?)),
			3u32 => Ok(Box::new(Add::deserialize(buf)?)),
			4u32 => Ok(Box::new(Sub::deserialize(buf)?)),
			5u32 => Ok(Box::new(Hp::deserialize(buf)?)),
			_ => Ok(Box::new(Const::deserialize(buf)?)),
		}
	}
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Const {
	pub value: u32,
}

impl Brick<u32> for Const {
	fn get_code(&self) -> u32 {
		return 0u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, _ctx: &mut Context) -> u32 {	
		return self.value
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Conditional {
	pub condition: Condition,
	pub positive: Value,
	pub negative: Value,
}
impl Brick<u32> for Conditional {
	fn get_code(&self) -> u32 {
		return 1u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> u32 {
		let cond = self.condition.run(ctx);
		if cond {
			self.positive.run(ctx)
		} else {
			self.negative.run(ctx)
		}
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Add {
	pub left: Value,
	pub right: Value,
}

impl Brick<u32> for Add {
	fn get_code(&self) -> u32 {
		return 2u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> u32 {	
		return self.left.run(ctx) + self.right.run(ctx);
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Sub {
	pub left: Value,
	pub right: Value,
}

impl Brick<u32> for Sub {
	fn get_code(&self) -> u32 {
		return 3u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> u32 {	
		return self.left.run(ctx) - self.right.run(ctx);
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Hp {
	pub object_index: u32,
}

impl Brick<u32> for Hp {
	fn get_code(&self) -> u32 {
		return 4u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> u32 {	
		return ctx.objects[self.object_index as usize].hp;
	}	
}

