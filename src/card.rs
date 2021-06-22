 use borsh::{BorshDeserialize, BorshSerialize};
use std::marker::Copy;
use solana_program::{
    pubkey::Pubkey,
    msg,
};
use crate::brick::BorshResult;
use crate::board::Place;
use std::io::Write;

#[derive(Debug, Clone, Copy)]
pub struct Card { //40
	pub id: u32, // 4
	pub card_type: Pubkey, // 32
	pub place: Place, //1
}

impl BorshSerialize for Card {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		self.id.serialize(writer);
		self.card_type.to_bytes().serialize(writer);
		self.place.serialize(writer);
		Ok(())
	}
}

impl BorshDeserialize for Card {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> {
		let id = u32::deserialize(buf)?;
		let card_type = Pubkey::new(&<[u8; 32]>::deserialize(buf)?);
		let place = Place::deserialize(buf)?;
		Ok(Card {
			id,
			card_type,
			place,
		})
	}
}