use std::result;
use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt::Debug;
use crate::unit::Unit;

pub type Action = Box<dyn Brick<()>>;
pub type Condition = Box<dyn Brick<bool>>;
pub type Value = Box<dyn Brick<u32>>;

pub type BorshResult<T> = result::Result<T, std::io::Error>;

// pub trait ContextObject {
// 	fn damage(&mut self, amount: u32) -> ();
// 	fn heal(&mut self, amount: u32) -> ();
// }

pub struct Context<'a> {
	pub objects: &'a mut Vec<Unit>,
}

pub trait Brick<T> where Self: Debug {
	fn get_code(&self) -> u32;
	fn b_to_vec(&self) -> Vec<u8>;
	fn run(&mut self, ctx: &mut Context) -> T;
}
