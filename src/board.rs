use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::{
	pubkey::Pubkey,
};
use crate::brick::BorshResult;
use crate::card::{
	Card,
	CardType
};
use crate::rand::Rand;
use crate::player::Player;
use std::cell::RefCell;
use std::io::Write;
use crate::brick::Context;
use std::collections::BTreeMap;

use std::rc::{Rc};




#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Log { //TODO: rename
	pub nonce: u32,
	pub message_len: u32,
	pub message: [u8; 128], //FIXME: should be replaced by smallvec or something similar
}

#[derive(Debug)]
pub struct Board { // 2536
	pub last_update: u64,
	pub step: u32,
	pub players: Vec<Rc<RefCell<Player>>>, //4 + 44 * 2
	pub card_types: Vec<Rc<RefCell<CardType>>>,
	pub cards: Vec<Rc<RefCell<Card>>>, //4 + 37 * 61
	pub log: Rc<RefCell<Log>>,
	pub rand: Rc<RefCell<Rand>>,
}

impl Board {
	pub fn cast_card(&self, card_id: u32, caster_id: u32) {
		let card = self.get_card_by_id(card_id);
		let card_type_id = card.unwrap().borrow().card_type;
	    let card_type = self.get_card_type_by_id(card_type_id);
	    let action = &mut card_type.unwrap().borrow_mut().get_action();
	    let ctx: &mut Context = &mut Context{ 
	         object: self.get_card_by_id(card_id).unwrap(),
	         board: &self,
	         caster_id: caster_id, //TODO: to vars
	         vars: BTreeMap::new(),
	    };
	    action.run(ctx);
	}
}

impl BorshSerialize for Board {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		self.last_update.serialize(writer)?;
		self.step.serialize(writer)?;
		(self.players.len() as u32).serialize(writer)?;
		for player in self.players.iter() {
			player.borrow().serialize(writer)?;
		}
		(self.card_types.len() as u32).serialize(writer)?;
		for card_type in self.card_types.iter() {
			card_type.borrow().serialize(writer)?;
		}
		(self.cards.len() as u32).serialize(writer)?;
		for card in self.cards.iter() {
			card.borrow().serialize(writer)?;
		}
		self.log.borrow().serialize(writer)?;
		self.rand.borrow().serialize(writer)?;
		Ok(())
	}
}

impl BorshDeserialize for Board {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> {
		let last_update = u64::deserialize(buf)?;
		let step = u32::deserialize(buf)?;
		let players_len = u32::deserialize(buf)?;
		let mut players = Vec::new();
		for _ in 0..players_len {
			let player = Player::deserialize(buf)?;
			players.push(Rc::new(RefCell::new(player)));
		}
		let card_types_len = u32::deserialize(buf)?;
		let mut card_types = Vec::new();
		for _ in 0..card_types_len {
			let card_type = CardType::deserialize(buf)?;
			card_types.push(Rc::new(RefCell::new(card_type)));
		}
		let cards_len = u32::deserialize(buf)?;
		let mut cards = Vec::new();
		for _ in 0..cards_len {
			let card = Card::deserialize(buf)?;
			cards.push(Rc::new(RefCell::new(card)));
		}
		let log = Rc::new(RefCell::new(Log::deserialize(buf)?));
		let rand = Rc::new(RefCell::new(Rand::deserialize(buf)?));
		Ok(Board {
			last_update,
			step,
			players,
			card_types,
			cards,
			log,
			rand,
		})
	}
}

impl Board{

	pub fn get_card_by_id(&self, id: u32) -> Option<Rc<RefCell<Card>>> {
		for card in self.cards.iter() {
			if card.borrow().id == id {
				return Some(Rc::clone(&card))
			}
		}
		return None
	}

	pub fn get_card_type_by_id(&self, id: u32) -> Option<Rc<RefCell<CardType>>> {
		for card_type in &self.card_types {
			if card_type.borrow().id == id {
				return Some(Rc::clone(&card_type))
			}
		}
		return None
	}

	pub fn get_cards_by_place(&self, place: u32) -> Vec<Rc<RefCell<Card>>> {
		let mut res = Vec::new();
		for card in self.cards.iter() {
			if card.borrow().place == place {
				res.push(Rc::clone(&card));
			}
		}
		return res
	}

	pub fn get_player(&self, index: u32) -> Option<Rc<RefCell<Player>>> { // gets player by id
		if self.players.len() < index as usize {
			return None
		}
		return Some(Rc::clone(&self.players[(index - 1) as usize]))
	}

	pub fn get_player_index_by_id(&self, id: Pubkey) -> u32 { // TODO: get_player_by_key
		for i in 0..self.players.len() {
			if self.players[i].borrow().id == id {
				return i as u32 + 1
			}
		}
		return 0
	}

	pub fn get_player_by_id(&self, id: Pubkey) -> Option<Rc<RefCell<Player>>> { // TODO: get_player_by_key
		for player in &self.players {
			if player.borrow().id == id {
				return Some(Rc::clone(&player))
			}
		}
		return None
	}
}
