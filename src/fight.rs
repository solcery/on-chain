use borsh::{BorshDeserialize, BorshSerialize};
use crate::unit::Unit;
use solana_program::{
    pubkey::Pubkey,
    msg,
};
use crate::brick::BorshResult;
use std::collections::HashMap;
use crate::board::PlaceId;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

#[derive(Debug)]
pub struct Fight {
	pub owner: Pubkey, //32
}

impl BorshSerialize for Fight {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		self.owner.to_bytes().serialize(writer);
		Ok(())
	}
}

impl BorshDeserialize for Fight {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> {
		let owner = Pubkey::new(&<[u8; 32]>::deserialize(buf)?);
		Ok(Fight{ owner })
	}
}

impl Fight{
	pub fn new(owner: Pubkey) -> Fight {
		return Fight { 
			owner: owner,
		}
	}
}
