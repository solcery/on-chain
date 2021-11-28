use crate::board::Board;
use crate::error::VMError;
use crate::rom::{CardTypesRom, Rom};
use crate::instruction_rom::InstructionRom;
use crate::vm::{SingleExecutionResult, VM};
use crate::word::Word;
use bincode;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
};
use std::convert::TryInto;

// It is a temporary solution. Later we'll support running action in multiple executions
const MAX_NUM_OF_VM_INSTRUCTION: usize = 1_000;

pub enum VMInstruction {
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person performing the move
    /// 1. `[]` ROM account
    /// 2. `[writable]` Board account
    ProcessAction {
        cardtype_index: u32,
        action_index: u32,
        args: Vec<Word>,
    },
}

impl VMInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input
            .split_first()
            .ok_or(ProgramError::InvalidInstructionData)?;

        // TODO: More descriptive error variants
        if rest.len() >= 8 {
            if *tag == 0 {
                let (cardtype_index_bytes, rest) = rest.split_at(4);
                let (action_index_bytes, rest) = rest.split_at(4);
                let args: Vec<Word> =
                    bincode::deserialize(rest).map_err(|_| ProgramError::InvalidInstructionData)?;
                let action_index = u32::from_le_bytes(action_index_bytes.try_into().unwrap());
                let cardtype_index = u32::from_le_bytes(cardtype_index_bytes.try_into().unwrap());

                Ok(Self::ProcessAction {
                    cardtype_index,
                    action_index,
                    args,
                })
            } else {
                Err(ProgramError::InvalidInstructionData)
            }
        } else {
            Err(ProgramError::InvalidInstructionData)
        }
    }
    pub fn process_instruction(&self, accounts: &[AccountInfo]) -> ProgramResult {
        match self {
            Self::ProcessAction {
                cardtype_index,
                action_index,
                args,
            } => {
                let account_info_iter = &mut accounts.iter();
                let initializer = next_account_info(account_info_iter)?;

                if !initializer.is_signer {
                    return Err(ProgramError::MissingRequiredSignature);
                }

                let rom_account = next_account_info(account_info_iter)?;
                let rom: Rom = rom_account
                    .deserialize_data()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                let board_account = next_account_info(account_info_iter)?;
                //Actually, here we should first transfer ownership of the board account to our
                //program, so  we can modify it.
                let mut board: Board = board_account
                    .deserialize_data()
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                //TODO: change this to actual Instructions and CardTypes accounts
                let instructions = unsafe { InstructionRom::from_raw_parts(&rom.instructions) };
                let card_types = unsafe { CardTypesRom::from_raw_parts(&rom.card_types) };

                let mut vm = VM::init_vm(
                    instructions,
                    card_types,
                    &mut board,
                    args,
                    *cardtype_index,
                    *action_index,
                );

                match vm.execute(MAX_NUM_OF_VM_INSTRUCTION) {
                    Ok(SingleExecutionResult::Finished) => {
                        drop(vm);
                        // As an optimization, we can use accounts, that store only the necessary
                        // amount of information. If this amount is exceeded, we should call
                        // SystemProgram::Allocate instruction, to change the size of the account.
                        board_account
                            .serialize_data(&board)
                            .map_err(|_| ProgramError::AccountDataTooSmall)?;
                        Ok(())
                    }
                    Ok(SingleExecutionResult::Unfinished) => {
                        Err(ProgramError::from(VMError::ComputationNotFinished))
                    }
                    Err(_) => {
                        unimplemented!();
                    }
                }
            }
        }
    }
}
