use crate::board::VerificationBoard;
use crate::card::CardType;
use crate::error::VMError;
use crate::instruction_rom::InstructionRom;
use crate::rom::CardTypesRom;
use crate::vm::{SingleExecutionResult, VM};
use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
};
use std::convert::TryFrom;

// It is a temporary solution. Later we'll support running action in multiple executions
const MAX_NUM_OF_VM_INSTRUCTION: usize = 1_000;

pub enum VMInstruction {
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person performing the move
    /// 1. `[]` CardTypes account
    /// 2. `[]` Instruction ROM account
    /// 3. `[writable]` Board account
    ProcessAction {
        cardtype_index: u32,
        action_index: u32,
        args: Vec<Word>,
    },
}

impl VMInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let mut data = input;
        let tag: u8 = BorshDeserialize::deserialize(&mut data).map_err(ProgramError::from)?;

        // TODO: More descriptive error variants
        if data.len() >= 8 {
            if tag == 0 {
                let cardtype_index: u32 =
                    BorshDeserialize::deserialize(&mut data).map_err(ProgramError::from)?;
                let action_index: u32 =
                    BorshDeserialize::deserialize(&mut data).map_err(ProgramError::from)?;
                let args: Vec<Word> =
                    BorshDeserialize::deserialize(&mut data).map_err(ProgramError::from)?;

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

                let card_types_account = next_account_info(account_info_iter)?;
                let card_types_data = card_types_account.data.borrow();
                let card_types: Vec<CardType> =
                    BorshDeserialize::deserialize(&mut card_types_data.as_ref())
                        .map_err(ProgramError::from)?;
                // TODO: Refactor this!
                let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

                let instructions_account = next_account_info(account_info_iter)?;
                let instructions_data = instructions_account.data.borrow();
                let instructions = InstructionRom::try_from(instructions_data.as_ref())
                    .map_err(|_| ProgramError::AccountDataTooSmall)?;

                let board_account = next_account_info(account_info_iter)?;
                let account_len = board_account.data_len(); // We need this to later reconstruct account size

                //Actually, here we should first transfer ownership of the board account to our
                //program, so  we can modify it.
                let board_data = board_account.data.borrow();
                let mut board: VerificationBoard =
                    BorshDeserialize::deserialize(&mut board_data.as_ref())
                        .map_err(ProgramError::from)?;
                drop(board_data);

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
                        let mut board_data = board_account.data.borrow_mut();
                        let mut serialized = board.try_to_vec().map_err(ProgramError::from)?;

                        if account_len >= serialized.len() {
                            // Accounts should have fixed size, thus here we extend the serialized
                            // data to the previous size
                            serialized
                                .extend(std::iter::repeat(0).take(account_len - serialized.len()));
                            board_data.copy_from_slice(&serialized);
                            Ok(())
                        } else {
                            unimplemented!();
                        }
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
