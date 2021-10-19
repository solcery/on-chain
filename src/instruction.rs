use crate::board::Board;
use crate::error::VMError;
use crate::rom::Rom;
use crate::vm::VM;
use crate::word::Word;
use flexbuffers::{FlexbufferSerializer, Reader};
use serde::{Deserialize, Serialize};
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
        card_index: u32,
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
                let (card_index_bytes, rest) = rest.split_at(4);
                let (action_index_bytes, rest) = rest.split_at(4);
                let reader =
                    Reader::get_root(rest).map_err(|_| ProgramError::InvalidInstructionData)?;

                let args = Vec::<Word>::deserialize(reader)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let action_index = u32::from_le_bytes(action_index_bytes.try_into().unwrap());
                let card_index = u32::from_le_bytes(card_index_bytes.try_into().unwrap());

                Ok(Self::ProcessAction {
                    card_index,
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
                card_index,
                action_index,
                args,
            } => {
                let account_info_iter = &mut accounts.iter();
                let initializer = next_account_info(account_info_iter)?;

                if !initializer.is_signer {
                    return Err(ProgramError::MissingRequiredSignature);
                }

                let rom_account = next_account_info(account_info_iter)?;
                let rom_bytes: &[u8] = &(*rom_account.data).borrow();
                let rom_reader =
                    Reader::get_root(rom_bytes).map_err(|_| ProgramError::InvalidAccountData)?;
                let rom =
                    Rom::deserialize(rom_reader).map_err(|_| ProgramError::InvalidAccountData)?;

                let board_account = next_account_info(account_info_iter)?;
                let board_bytes: &[u8] = &(*board_account.data).borrow();
                let board_reader =
                    Reader::get_root(board_bytes).map_err(|_| ProgramError::InvalidAccountData)?;
                let mut board = Board::deserialize(board_reader)
                    .map_err(|_| ProgramError::InvalidAccountData)?;

                let mut vm = VM::init_vm(&rom, &mut board, args, *card_index, *action_index);

                vm.execute(MAX_NUM_OF_VM_INSTRUCTION);

                if vm.is_halted() {
                    drop(vm);
                    let mut board_serializer = FlexbufferSerializer::new();
                    rom.serialize(&mut board_serializer).unwrap();
                    let mut board_bytes = (*board_account.data).borrow_mut();
                    // DANGER: This will likely panic in runtime, as this function panics, if
                    // slices have different length.
                    board_bytes.clone_from_slice(board_serializer.view());
                    Ok(())
                } else {
                    Err(ProgramError::from(VMError::ComputationNotFinished))
                }
            }
        }
    }
}
