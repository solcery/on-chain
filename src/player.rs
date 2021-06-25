use borsh::{BorshDeserialize, BorshSerialize};
use std::marker::Copy;
use solana_program::{
    pubkey::Pubkey,
};
use crate::brick::BorshResult;
use std::io::Write;

#[derive(Debug, Clone, Copy)]
pub struct Player { //4
	pub id: Pubkey, //32
	pub attrs: [i32; 3],  //4 * 3
}

impl BorshSerialize for Player {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		self.id.to_bytes().serialize(writer);
		self.attrs.serialize(writer);
		Ok(())
	}
}

impl BorshDeserialize for Player {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> {
		let id = Pubkey::new(&<[u8; 32]>::deserialize(buf)?);
		let attrs = <[i32;3]>::deserialize(buf)?;
		Ok(Player {
			id,
			attrs,
		})
	}
}