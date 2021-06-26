use crate::brick::{ 
	Context, Brick, BorshResult, Action, Condition, Value
};
use solana_program::{
    pubkey::Pubkey,
};
use std::io::Write;
use borsh::{
	BorshDeserialize, BorshSerialize
};
use std::convert::TryInto;
use std::rc::Rc;
use crate::board::Place;
use crate::player::Player;
use crate::rand::Rand;
use std::cmp;
use std::collections::BTreeMap;

impl BorshSerialize for Action {
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

impl BorshDeserialize for Action {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> { 
		let _action_code = u32::from_le_bytes(buf[..4].try_into().unwrap());
		*buf = &buf[4..];
		let code = u32::from_le_bytes(buf[..4].try_into().unwrap());
		*buf = &buf[4..];
		match code {
			0u32 => Ok(Box::new(Void::deserialize(buf)?)),
			1u32 => Ok(Box::new(Set::deserialize(buf)?)),
			2u32 => Ok(Box::new(Conditional::deserialize(buf)?)),
			3u32 => Ok(Box::new(Loop::deserialize(buf)?)),
			4u32 => Ok(Box::new(Card::deserialize(buf)?)),
			100u32 => Ok(Box::new(MoveTo::deserialize(buf)?)),
			101u32 => Ok(Box::new(SetPlayerAttr::deserialize(buf)?)),
			102u32 => Ok(Box::new(AddPlayerAttr::deserialize(buf)?)),
			103u32 => Ok(Box::new(ApplyToPlace::deserialize(buf)?)),
			_ => Ok(Box::new(Void{})),
		}
	}
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Void {}

impl Brick<()> for Void {
	fn get_code(&self) -> u32 {
		return 0u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, _ctx: &mut Context) -> () {}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Set{
	pub action1: Action,
	pub action2: Action,
}
impl Brick<()> for Set {
	fn get_code(&self) -> u32 {
		return 1u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () {
		self.action1.run(ctx);
		self.action2.run(ctx);
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
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

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Loop {
	pub iterations: Value,
	pub action: Action,
}
impl Brick<()> for Loop {
	fn get_code(&self) -> u32 {
		return 3u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () {
		let iterations = self.iterations.run(ctx);
		for i in 1..iterations {
			self.action.run(ctx);
		}
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Card {
	pub card_type: u32,
}
impl Brick<()> for Card {
	fn get_code(&self) -> u32 {
		return 4u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () {
		let mut card_type = ctx.board.get_card_type_by_id(self.card_type);
		let mut action = card_type.unwrap().borrow_mut().get_action(); 
		action.run(ctx);
	}	
}



#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct MoveTo {
	pub place: Value,
}
impl Brick<()> for MoveTo {
	fn get_code(&self) -> u32 {
		return 100u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () {
		let place = self.place.run(ctx);
		let mut card = ctx.object.borrow_mut();
		card.place = Place::from_i32(place);
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SetPlayerAttr {
	pub attr_index: u32,
	pub player_index: Value,
	pub attr_value: Value,
}
impl Brick<()> for SetPlayerAttr {
	fn get_code(&self) -> u32 {
		return 101u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () {
		let player_index = self.player_index.run(ctx);
		let attr_value = self.attr_value.run(ctx);
		let player = ctx.board.get_player(player_index.try_into().unwrap());
		player.unwrap().borrow_mut().attrs[self.attr_index as usize] = attr_value;
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct AddPlayerAttr {
	pub attr_index: u32,
	pub player_index: Value,
	pub attr_value: Value,
}
impl Brick<()> for AddPlayerAttr {
	fn get_code(&self) -> u32 {
		return 102u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () {
		let player_index = self.player_index.run(ctx);
		let attr_value = self.attr_value.run(ctx);
		let player = ctx.board.get_player(player_index.try_into().unwrap());
		player.unwrap().borrow_mut().attrs[self.attr_index as usize] += attr_value;
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct ApplyToPlace {
	pub place: Value,
	pub action: Action,
	pub limit: Value,
}
impl Brick<()> for ApplyToPlace {
	fn get_code(&self) -> u32 {
		return 103u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () { // JUST TOO MUCH
		let place = self.place.run(ctx);
		let mut limit = self.limit.run(ctx);
		let mut rng = Rand::new(0);
		let mut cards = ctx.board.get_cards_by_place(Place::from_i32(place));
		if limit == 0 {
			limit = cards.len().try_into().unwrap();
		}
		let old_object = Rc::clone(&ctx.object);
		for i in 0..limit {
			rng.shuffle(&mut cards);
			let new_object = Rc::clone(&cards.pop().unwrap());
			ctx.object = new_object;
			self.action.run(ctx);
		}
		ctx.object = old_object;
	}	
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct SetCtxVar{
	pub var_index: u32,
	pub value: Value,
}
impl Brick<()> for SetCtxVar {
	fn get_code(&self) -> u32 {
		return 104u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> () {
		let value = self.value.run(ctx);
		ctx.vars.insert(self.var_index, value);
	}	
}

