use crate::brick::{ Context, Brick, BorshResult, Action, Condition, Value};
use std::io::Write;
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryInto;

#[repr(u32)]
enum Actions { //TODO
	Void = 0u32,
	Set,
	Conditional,
	
	Damage,
	Heal,
}

impl BorshSerialize for Action {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		let action_code = 0u32.to_le_bytes();
		let code = self.get_code();
		writer.write_all(&action_code);
		writer.write_all(&code.to_le_bytes());
		let x = self.b_to_vec();
		writer.write_all(&x);
		Ok(())
	}
}

impl BorshDeserialize for Action {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> { 
		let action_code = u32::from_le_bytes(buf[..4].try_into().unwrap());
		*buf = &buf[4..];
		let code = u32::from_le_bytes(buf[..4].try_into().unwrap());
		*buf = &buf[4..];
		match code {
			0u32 => Ok(Box::new(Void::deserialize(buf)?)),
			1u32 => Ok(Box::new(Set::deserialize(buf)?)),
			2u32 => Ok(Box::new(Conditional::deserialize(buf)?)),
			3u32 => Ok(Box::new(Damage::deserialize(buf)?)),
			4u32 => Ok(Box::new(Heal::deserialize(buf)?)),
			_ => Ok(Box::new(Damage::deserialize(buf)?)),
		}
	}
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Void {}

impl Brick<()> for Void {
	fn get_code(&self) -> u32 {
		return 0u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () {}	
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Set {
	pub impacts: Vec<Action>,
}
impl Brick<()> for Set {
	fn get_code(&self) -> u32 {
		return 1u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () {
		let impact_iter = self.impacts.iter_mut();
		for mut impact in impact_iter {
			impact.run(ctx);
		}
	}	
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Conditional {
	pub condition: Condition,
	pub positive: Action,
	pub negative: Action,
}
impl Brick<()> for Conditional {
	fn get_code(&self) -> u32 {
		return 2u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () {
		let cond = self.condition.run(ctx);
		if cond {
			self.positive.run(ctx)
		} else {
			self.negative.run(ctx)
		}
	}	
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Damage {
	pub amount: Value,
}
impl Brick<()> for Damage {
	fn get_code(&self) -> u32 {
		return 3u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () {
		let amount = self.amount.run(ctx);
		ctx.obj.damage(amount)
	}	
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Heal {
	pub amount: Value,
}
impl Brick<()> for Heal {
	fn get_code(&self) -> u32 {
		return 4u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () {
		let amount = self.amount.run(ctx);
		ctx.obj.heal(amount)
	}	
}


