use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::convert::{
	TryInto,
	TryFrom,
};
use num_enum::TryFromPrimitive;

pub struct Unit {
	pub hp: u32,
	pub damage: u32,
}

pub struct Context {
	pub obj: Unit,
}

pub trait Impact {
	fn test_run(&mut self) -> bool { return true }
}

//trait Unpackable {
//	fn unpack_from_slice(src: &[u8]) -> Result<Box<Self>, ProgramError>;
//}

pub fn pop_u32(mut input: &mut &[u8]) -> Result<u32, ProgramError> {
    let amount = input
        .get(..4) //get 4 elements from array
        .and_then(|slice| slice.try_into().ok()) // turning it into slice?
        .map(u32::from_be_bytes) // and into u64??
        .ok_or(ProgramError::IncorrectProgramId)?;
    *input = &input[4..];
    Ok(amount)
}

pub struct DealDamage {
	pub amount: u32,
}
impl DealDamage {
	fn unpack_from_slice(mut src: &mut &[u8]) -> Result<Box<dyn Impact>, ProgramError> {
		let amount = pop_u32(src)?;
		Ok(Box::new(DealDamage{ amount }))
	}
}
impl Impact for DealDamage {
	fn test_run(&mut self) -> bool {
		msg!("executing DealDamage with damage = {}", self.amount);
		return true
	}	
}

pub struct Heal {
	pub amount: u32,
}
impl Impact for Heal {
	fn test_run(&mut self) -> bool {
		msg!("executing Heal with power = {}", self.amount);
		return true
	}
}
impl Heal {
	fn unpack_from_slice(mut src: &mut &[u8]) -> Result<Box<dyn Impact>, ProgramError> {
		let amount = pop_u32(src)?;
		Ok(Box::new(Heal{ amount }))
	}
}

pub struct Set {
	pub impacts: Vec<Box<dyn Impact>>,
}
impl Impact for Set {
	fn test_run(&mut self) -> bool {
		msg!("executing Set:");
		let impact_iter = self.impacts.iter_mut();
		for mut impact in impact_iter {
			impact.test_run();
		}
		return true
	}
}
impl Set {
	fn unpack_from_slice(mut src: &mut &[u8]) -> Result<Box<dyn Impact>, ProgramError> {
		let amount = pop_u32(src)?;
		let mut impacts = Vec::new();
        	for _n in 1..amount + 1 {
        		impacts.push(unpack_impact(src)?);
        	}
		Ok(Box::new(Set{ impacts }))
	}
}
#[repr(u32)]
#[derive(TryFromPrimitive)]
pub enum ImpactType {
	#[num_enum(default)]
	Void = 0,
	Set,
	DealDamage,
	Heal,
}


pub fn unpack_impact(mut data: &mut &[u8]) -> Result<Box<dyn Impact>, ProgramError> {
	let impact_type = pop_u32(data)?;
	let impact_type = ImpactType::try_from(impact_type);
	match impact_type {
		Ok(ImpactType::Set) => Set::unpack_from_slice(data),
		Ok(ImpactType::DealDamage) => DealDamage::unpack_from_slice(data),
		Ok(ImpactType::Heal) => Heal::unpack_from_slice(data),
		_ => Err(ProgramError::IncorrectProgramId),
	}
}


entrypoint!(process_instruction);
pub fn process_instruction(
    _program_id: &Pubkey, // Public key of the account the program was loaded into
    _accounts: &[AccountInfo], // The account to store number in
    instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    
    
    let mut data = &instruction_data[..]; // Copying instruction_data to mutable slice
    let mut impact3: Box<dyn Impact> = unpack_impact(&mut data)?;
    impact3.test_run();
    Ok(())
}





/*
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::convert::TryInto;


#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Unit {
	pub hp: u32,
	pub damage: u32,
}

pub struct Context {
	pub obj: Unit,
}

trait Impact {
	fn run (&mut self, ctx: Context) -> bool;
	fn unpack_from_slice(src: &[u8]) -> Result<Box<Self>, ProgramError>;
}

pub struct DealDamage {
	pub amount: u32,
}
impl Impact for DealDamage {
	fn run (&mut self, ctx: Context) -> bool {
		ctx.obj.hp -= self.amount;
		return true
	}
	fn unpack_from_slice(src: &[u8]) -> Box<Self> {
		let amount = src
        	.get(..4) //get 4 elements from array
        	.and_then(|slice| slice.try_into().ok()) // turning it into slice?
        	.map(u32::from_be_bytes) // and into u64??
        	.ok_or(ProgramError::IncorrectProgramId)?;
		Ok(Box::new(DealDamage{ amount }))
	}	
}

pub struct Heal {
	pub amount: u32,
}
impl Impact for Heal {
	fn run (&mut self, ctx: Context) -> bool {
		ctx.obj.hp += self.amount;
		return true
	}
	fn unpack_from_slice(src: &[u8]) -> Box<Self> {
		let amount = src
        	.get(..4) //get 4 elements from array
        	.and_then(|slice| slice.try_into().ok()) // turning it into slice?
        	.map(u32::from_be_bytes) // and into u64??
        	.ok_or(ProgramError::IncorrectProgramId)?;
		Ok(Box::new(Heal{ amount }))
	}	
}

pub struct DealSelfDamage {}
impl Impact for DealSelfDamage {
	fn run (&mut self, ctx: Context) -> bool {
		ctx.obj.hp -= ctx.obj.damage;
		return true
	}
	fn unpack_from_slice(src: &[u8]) -> Result<Box<Self>, ProgramError> {
		Ok(Box::new(DealSelfDamage{}))
	}	
}

pub enum ImpactType {
    DealDamage,
    Heal,
    DealSelfDamage,
}

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey, // Public key of the account the program was loaded into
    accounts: &[AccountInfo], // The account to store number in
    instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    // Get the account to say hello to
    let account = next_account_info(accounts_iter)?;

    // The account must be owned by the program in order to modify its data
    if account.owner != program_id {
        msg!("Modified account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Increment and store the number of times the account has been greeted
    let (impact_type, data) = instruction_data.split_first().ok_or(ProgramError::IncorrectProgramId)?;
    
    let impact: Box<dyn Impact> = match impact_type {
    	0 => DealDamage::unpack_from_slice(data)?, //TODO: add error handling
        1 => Heal::unpack_from_slice(data),
        2 => DealSelfDamage::unpack_from_slice(data),
    };
    let mut unit = Unit::try_from_slice(&account.data.borrow())?;
    let mut ctx = Context {
    	obj: unit,
    };
    Ok(impact.run(ctx))
}

fn unpack_number(input: &[u8]) -> Result<u32, ProgramError> {
    let amount = input
        .get(..4) //get 4 elements from array
        .and_then(|slice| slice.try_into().ok()) // turning it into slice?
        .map(u32::from_be_bytes) // and into u64??
        .ok_or(ProgramError::IncorrectProgramId)?;
    Ok(amount)
}
*/
