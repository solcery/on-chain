use std::result;
use std::fmt::Debug;
use crate::board::Board;
use crate::card::Card;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

pub type Action = Box<dyn Brick<()>>;
pub type Condition = Box<dyn Brick<bool>>;
pub type Value = Box<dyn Brick<i32>>;

pub type BorshResult<T> = result::Result<T, std::io::Error>;

pub struct Context {
	pub object: Rc<RefCell<Card>>,
	pub board: Board,
}

pub trait Brick<T> where Self: Debug {
	fn get_code(&self) -> u32;
	fn b_to_vec(&self) -> Vec<u8>;
	fn run(&mut self, ctx: &mut Context) -> T;
}
