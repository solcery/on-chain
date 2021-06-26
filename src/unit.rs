use borsh::{BorshDeserialize, BorshSerialize};
use std::marker::Copy;
use solana_program::{
    pubkey::Pubkey,
};
use crate::brick::BorshResult;
use crate::board::PlaceId;
use std::io::Write;

#[derive(Debug, Clone, Copy)]
pub struct Unit {
	pub owner: Pubkey,
	pub unit_type: Pubkey, //32
	pub place: PlaceId,
	pub hp: u32, //4
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct UnitType {
	pub hp: u32,
}

impl BorshSerialize for Unit {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		self.owner.to_bytes().serialize(writer);
		self.place.serialize(writer);
		self.unit_type.to_bytes().serialize(writer);
		self.hp.serialize(writer);
		Ok(())
	}
}

impl BorshDeserialize for Unit {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> {
		let owner = Pubkey::new(&<[u8; 32]>::deserialize(buf)?);
		let place = <(u32, u32)>::deserialize(buf)?;
		let unit_type = Pubkey::new(&<[u8; 32]>::deserialize(buf)?);
		let hp = u32::deserialize(buf)?;
		Ok(Unit {
			owner,
			hp,
			unit_type,
			place,
		})
	}
}