use crate::brick::{ Context, Brick, BorshResult, Value};
use std::io::Write;
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryInto;

impl BorshSerialize for Value {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		let value_code = 2u32.to_le_bytes();
		let code = self.get_code();
		writer.write_all(&value_code);
		writer.write_all(&code.to_le_bytes());
		let x = self.b_to_vec();
		writer.write_all(&x);
		Ok(())
	}
}

impl BorshDeserialize for Value {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> { 
		let value_code = u32::from_le_bytes(buf[..4].try_into().unwrap());
		*buf = &buf[4..];
		let code = u32::from_le_bytes(buf[..4].try_into().unwrap());
		*buf = &buf[4..];
		match code {
			0u32 => Ok(Box::new(Const::deserialize(buf)?)),
			_ => Ok(Box::new(Const::deserialize(buf)?)),
		}
	}
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Const {
	pub value: u32,
}

impl Brick<u32> for Const {
	fn get_code(&self) -> u32 {
		return 0u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, _ctx: &mut Context) -> u32 {	
		return self.value
	}	
}

