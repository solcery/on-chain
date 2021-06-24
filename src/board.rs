use borsh::{BorshDeserialize, BorshSerialize};
use std::marker::Copy;
use solana_program::{
    pubkey::Pubkey,
    declare_id,
    msg,
};
use crate::brick::BorshResult;
use crate::card::Card;
use crate::rand::Rand;
use crate::player::Player;
use std::cell::RefCell;
use std::io::Write;
use std::cmp::PartialEq;
use std::rc::{Rc, Weak};


declare_id!("A1U9yQfGgNMn2tkE5HB576QYoBA3uAdNFdjJA439S4m6");

#[derive(Debug, Copy, Clone, BorshSerialize, BorshDeserialize, PartialEq)]
pub enum Place { //4
    Nowhere,
    Deck,
    Shop,
    Hand1,
    Hand2,
    DrawPile1,
    DrawPile2,
}

impl Place {
    pub fn from_u8(value: u8) -> Place {
        match value {
            1 => Place::Deck,
            2 => Place::Shop,
            3 => Place::Hand1,
            4 => Place::Hand2,
            5 => Place::DrawPile1,
            6 => Place::DrawPile2,
            _ => Place::Nowhere,
        }
    }

    pub fn from_i32(value: i32) -> Place {
    	match value {
            1 => Place::Deck,
            2 => Place::Shop,
            3 => Place::Hand1,
            4 => Place::Hand2,
            5 => Place::DrawPile1,
            6 => Place::DrawPile2,
            _ => Place::Nowhere,
        }
    }
}

#[derive(Debug)]
pub struct Board { // 2536
	pub players: Vec<Rc<RefCell<Player>>>, //4 + 44 * 2
	pub cards: Vec<Rc<RefCell<Card>>>, //4 + 37 * 61
}

#[derive(Debug)]
pub struct Ruleset {
	pub deck: Vec<(Pubkey, u32, Place)>,
}

impl BorshSerialize for Board {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		(self.players.len() as u32).serialize(writer);
		for player in self.players.iter() {
			player.borrow().serialize(writer);
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
		let cards_len = u32::deserialize(buf)?;
		let mut cards = Vec::new();
		for i in 0..cards_len {
			let card = Card::deserialize(buf)?;
			cards.push(Rc::new(RefCell::new(card)));
		}
		Ok(Board {
			players: players,
			cards: cards,
		})
	}
}

impl Board{
	pub fn new(ruleset: Ruleset) -> Board {
		let mut cards = Vec::new();
		let mut card_id = 0;
		for card_type in ruleset.deck.iter() {
			for i in 0..card_type.1 {
				cards.push(Rc::new(RefCell::new(Card {
					id: card_id,
					card_type: card_type.0,
					place: card_type.2,
				})));
				card_id += 1;
			}
		}
		let mut rng = Rand::new(0);
		rng.shuffle(&mut cards);
		let mut dealt_cards = Vec::new();
		for card in &cards {
			dealt_cards.push(Rc::clone(&card));
		}
		for i in 1..6 {
			dealt_cards.pop().unwrap().borrow_mut().place = Place::Hand1;
			dealt_cards.pop().unwrap().borrow_mut().place = Place::Hand2;
		}
		for i in 1..6 {
			dealt_cards.pop().unwrap().borrow_mut().place = Place::DrawPile1;
			dealt_cards.pop().unwrap().borrow_mut().place = Place::DrawPile2;
		}
		for i in 1..6 {
			dealt_cards.pop().unwrap().borrow_mut().place = Place::Shop;
		}
		Board {
			cards: cards,
			players: Vec::new(),
		}
	}

	pub fn start(&self) {
		self.players[0].borrow_mut().attrs[0] = 1;
	}

	pub fn get_card_by_id(&self, id: u32) -> Option<Rc<RefCell<Card>>> {
		for card in &self.cards {
			if (card.borrow().id == id) {
				return Some(Rc::clone(&card))
			}
		}
		return None
	}

	pub fn get_cards_by_place(&self, place: Place) -> Vec<Rc<RefCell<Card>>> {
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

	pub fn get_player_by_id(&self, id: Pubkey) -> Option<Rc<RefCell<Player>>> {
		for player in &self.players {
			if player.borrow().id == id {
				return Some(Rc::clone(&player))
			}
		}
		return None
	}
}

// 	pub fn get_unit_by_place(&self, place_id: PlaceId) -> Option<Rc<RefCell<Unit>>> {
// 		for unit in self.units.iter() {
// 			if unit.borrow().place == place_id {
// 				return Some(Rc::clone(&unit))
// 			}
// 		}
// 		return None
// 	}

// 	pub fn get_unit_by_type(&self, unit_type: Pubkey) -> Option<Rc<RefCell<Unit>>> {
// 		for unit in self.units.iter() {
// 			if unit.borrow().unit_type == unit_type {
// 				return Some(Rc::clone(&unit))
// 			}
// 		}
// 		return None
// 	}

// 	pub fn get_unit_by_id(&self, id: u32) -> Option<Rc<RefCell<Unit>>> {
// 		return Some(Rc::clone(&self.units[id as usize]))
// 	}

// 	pub fn create_unit(&mut self, owner:Pubkey, unit_type: Pubkey, place: PlaceId) {
// 		let new_unit = Unit {
// 			owner,
// 			unit_type,
// 			place,
// 			hp: 20,
// 		};
// 		self.units.push(Rc::new(RefCell::new(new_unit)));
// 	}
// }


