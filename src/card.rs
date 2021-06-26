use borsh::{BorshDeserialize, BorshSerialize};
use std::marker::Copy;
use std::convert::TryInto;
use solana_program::{
	account_info::AccountInfo,
    pubkey::Pubkey,
    msg,
};
use crate::brick::{
	BorshResult, 
	Action
};
use crate::board::Place;
use std::io::Write;

#[derive(Debug, Clone, Copy, BorshDeserialize, BorshSerialize)]
pub struct Card { //9
	pub id: u32, // 4
	pub card_type: u32, // 4
	pub place: Place, //1
}

#[derive(Debug)]
pub struct CardType {
	pub id: u32,
	pub key: Pubkey,
	pub data: Vec<u8>,
}

impl CardType {
	pub fn new(id: u32, account_info: &AccountInfo) -> CardType {
		return CardType {
			id: id,
			key: *account_info.key,
			data: account_info.data.borrow()[..].to_vec(),
		}
	}

	pub fn get_action(&self) -> Action {
		let client_metadata_size = u32::from_le_bytes(self.data[..4].try_into().unwrap());
    	return Action::try_from_slice(&self.data[client_metadata_size as usize + 4..]).unwrap();    
	}

	pub fn get_client_metadata(&self) -> Vec<u8> {
		let client_metadata_size = u32::from_le_bytes(self.data[..4].try_into().unwrap());
    	return self.data[..client_metadata_size as usize + 4].to_vec();  
	}
}

impl BorshSerialize for CardType {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		self.id.serialize(writer);
		self.key.to_bytes().serialize(writer);
		self.data.serialize(writer);
		Ok(())
	}
}

impl BorshDeserialize for CardType {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> {
		let id = u32::deserialize(buf)?;
		let key = Pubkey::new(&<[u8; 32]>::deserialize(buf)?);
		let data = Vec::<u8>::deserialize(buf)?;
		Ok(CardType {
			id,
			key,
			data
		})
	}
}