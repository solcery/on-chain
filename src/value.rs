use crate::brick::{ Context, Brick, BorshResult, Value, Condition};
use std::io::Write;
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryInto;
use crate::board::Place;

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
			2u32 => Ok(Box::new(Add::deserialize(buf)?)),
			3u32 => Ok(Box::new(Sub::deserialize(buf)?)),
			100u32 => Ok(Box::new(GetPlayerAttr::deserialize(buf)?)),
			101u32 => Ok(Box::new(GetPlayerIndex::deserialize(buf)?)),
			102u32 => Ok(Box::new(GetCardsAmount::deserialize(buf)?)),
			103u32 => Ok(Box::new(CurrentPlace::deserialize(buf)?)),
			104u32 => Ok(Box::new(GetCtxVar::deserialize(buf)?)),
			105u32 => Ok(Box::new(CasterPlayerIndex::deserialize(buf)?)),
			_ => Ok(Box::new(Const { value: 0 })), // TODO Err
		}
	}
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Const {
	pub value: i32,
}

impl Brick<i32> for Const {
	fn get_code(&self) -> u32 {
		return 0u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, _ctx: &mut Context) -> i32 {	
		return self.value
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Conditional {
	pub condition: Condition,
	pub positive: Value,
	pub negative: Value,
}
impl Brick<i32> for Conditional {
	fn get_code(&self) -> u32 {
		return 1u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> i32 {
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

impl Brick<i32> for Add {
	fn get_code(&self) -> u32 {
		return 2u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> i32 {	
		return self.left.run(ctx) + self.right.run(ctx);
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Sub {
	pub left: Value,
	pub right: Value,
}

impl Brick<i32> for Sub {
	fn get_code(&self) -> u32 {
		return 3u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> i32 {	
		return self.left.run(ctx) - self.right.run(ctx);
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GetPlayerAttr {
	pub attr_index: u32,
	pub player_index: Value,
}

impl Brick<i32> for GetPlayerAttr {
	fn get_code(&self) -> u32 {
		return 100u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> i32 {
		let card_place = ctx.object.borrow().place;
		let player_index = self.player_index.run(ctx);
		let player = ctx.board.get_player(player_index.try_into().unwrap());
		return player.unwrap().borrow_mut().attrs[self.attr_index as usize];
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GetPlayerIndex {}

impl Brick<i32> for GetPlayerIndex {
	fn get_code(&self) -> u32 {
		return 101u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> i32 {
		let card_place = ctx.object.borrow().place;
		return match card_place {
			Place::Hand1 | Place::DrawPile1 => 1,
			Place::Hand2 | Place::DrawPile2 => 2,
			_ => 0,
		};
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GetCardsAmount {
	pub place: Value,
}
impl Brick<i32> for GetCardsAmount {
	fn get_code(&self) -> u32 {
		return 102u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> i32 {
		let place = self.place.run(ctx);
		return ctx.board.get_cards_by_place(Place::from_i32(place)).len() as i32;
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CurrentPlace {}

impl Brick<i32> for CurrentPlace {
	fn get_code(&self) -> u32 {
		return 103u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> i32 {
		return ctx.object.borrow().place as i32;
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GetCtxVar {
	pub var_index: u32,
}
impl Brick<i32> for GetCtxVar {
	fn get_code(&self) -> u32 {
		return 104u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> i32 {
		return *ctx.vars.get(&self.var_index).or(Some(&0)).unwrap();
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct CasterPlayerIndex {}

impl Brick<i32> for CasterPlayerIndex {
	fn get_code(&self) -> u32 {
		return 105u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> i32 {
		return ctx.caster_id as i32;
	}	
}

