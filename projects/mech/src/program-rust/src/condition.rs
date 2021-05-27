use crate::brick::{ Context, Brick, BorshResult, Condition};
use std::io::Write;
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryInto;

impl BorshSerialize for Condition {
	fn serialize<W: Write>(&self, writer: &mut W) -> BorshResult<()> {
		let condition_code = 1u32.to_le_bytes();
		let code = self.get_code();
		writer.write_all(&condition_code);
		writer.write_all(&code.to_le_bytes());
		let x = self.b_to_vec();
		writer.write_all(&x);
		Ok(())
	}
}

impl BorshDeserialize for Condition {
	fn deserialize(buf: &mut &[u8]) -> std::result::Result<Self, std::io::Error> { 
		let condition_code = u32::from_le_bytes(buf[..4].try_into().unwrap());
		*buf = &buf[4..];
		let code = u32::from_le_bytes(buf[..4].try_into().unwrap());
		*buf = &buf[4..];
		match code {
			1u32 => Ok(Box::new(True::deserialize(buf)?)),
			_ => Ok(Box::new(True::deserialize(buf)?)),
		}
	}
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct True {}

impl Brick<bool> for True {
	fn get_code(&self) -> u32 {
		return 0u32 
	}
	fn b_to_vec(&self) -> Vec<u8> {
		return self.try_to_vec().unwrap();
	}
	fn run(&mut self, ctx: &mut Context) -> bool {	
		return true
	}	
}

