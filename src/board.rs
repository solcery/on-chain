use borsh::{BorshDeserialize, BorshSerialize};
use std::marker::Copy;
use solana_program::{
	program_error::ProgramError,
	account_info::AccountInfo,
    pubkey::Pubkey,
    declare_id,
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
use std::cmp::PartialEq;
use std::rc::{Rc, Weak};
use crate::error::SolceryError;
use crate::ruleset::Ruleset;

declare_id!("5Ds6QvdZAqwVozdu2i6qzjXm8tmBttV6uHNg4YU8rB1P");


#[derive(Debug)]
pub struct Board { // 2536
	pub players: Vec<Rc<RefCell<Player>>>, //4 + 44 * 2
	pub card_types: Vec<Rc<RefCell<CardType>>>,
	pub cards: Vec<Rc<RefCell<Card>>>, //4 + 37 * 61
}

impl Board {
	pub fn cast_card(&self, card_id: u32, caster_id: u32) {
		let card = self.get_card_by_id(card_id);
		let card_type_id = card.unwrap().borrow().card_type;
	    let card_type = self.get_card_type_by_id(card_type_id);
	    let mut action = &mut card_type.unwrap().borrow_mut().get_action();
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
		(self.players.len() as u32).serialize(writer);
		for player in self.players.iter() {
			player.borrow().serialize(writer);
		}
		(self.card_types.len() as u32).serialize(writer);
		for card_type in self.card_types.iter() {
			card_type.borrow().serialize(writer);
		}
		(self.cards.len() as u32).serialize(writer);
		for card in self.cards.iter() {
			card.borrow().serialize(writer);
		}
		Ok(())
	}
}

impl BorshDeserialize for Board {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> {
		let players_len = u32::deserialize(buf)?;
		let mut players = Vec::new();
		for i in 0..players_len {
			let player = Player::deserialize(buf)?;
			players.push(Rc::new(RefCell::new(player)));
		}
		let card_types_len = u32::deserialize(buf)?;
		let mut card_types = Vec::new();
		for i in 0..card_types_len {
			let card_type = CardType::deserialize(buf)?;
			card_types.push(Rc::new(RefCell::new(card_type)));
		}
		let cards_len = u32::deserialize(buf)?;
		let mut cards = Vec::new();
		for i in 0..cards_len {
			let card = Card::deserialize(buf)?;
			cards.push(Rc::new(RefCell::new(card)));
		}
		Ok(Board {
			players,
			card_types,
			cards,
		})
	}
}

impl Board{

	pub fn start(&self) {
		for card in self.cards.iter() {
	        if (card.borrow().place == 0) {
	            self.cast_card(card.borrow().id, 0);
	        }
	    }
	}

	pub fn get_card_by_id(&self, id: u32) -> Option<Rc<RefCell<Card>>> {
		for card in self.cards.iter() {
			if (card.borrow().id == id) {
				return Some(Rc::clone(&card))
			}
		}
		return None
	}

	pub fn get_card_type_by_id(&self, id: u32) -> Option<Rc<RefCell<CardType>>> {
		for card_type in &self.card_types {
			if (card_type.borrow().id == id) {
				return Some(Rc::clone(&card_type))
			}
		}
		return None
	}

	pub fn get_card_type_by_key(&self, key: Pubkey) -> Option<Rc<RefCell<CardType>>> {
		for card_type in &self.card_types {
			if (card_type.borrow().key == key) {
				return Some(Rc::clone(&card_type))
			}
		}
		return None
	}

	pub fn get_cards_by_place(&self, place: u32) -> Vec<Rc<RefCell<Card>>> {
		let mut res = Vec::new();
		for card in self.cards.iter() {
			if (card.borrow().place == place) {
				res.push(Rc::clone(&card));
			}
		}
		return res
	}

	pub fn get_player(&self, index: u32) -> Option<Rc<RefCell<Player>>> {
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
