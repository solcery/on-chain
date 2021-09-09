use std::marker::Copy;

#[derive(Debug, Clone, Copy)]
pub struct Card { //9
	pub id: u32, // 4
	pub card_type: u32, // 4
	pub place: u32, //4
}

#[derive(Debug)]
pub struct CardType {
	pub id: u32,
	pub data: Vec<u8>,
}

impl CardType {
}
