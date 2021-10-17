use crate::word::Word;
use flexbuffers::Reader;
use serde::{Deserialize, Serialize};
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

pub enum VMInstruction {
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person performing the move
    /// 1. `[]` ROM account
    /// 2. `[writable]` Board account
    ProcessAction {
        card_index: u32,
        entrypoint_index: u32,
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
            if tag == 0 {
                let (card_index_bytes, rest) = rest.split_at(4);
                let (entrypoint_index_bytes, rest) = rest.split_at(4);
                let reader =
                    Reader::get_root(rest).map_err(|_| ProgramError::InvalidInstructionData)?;

                let args = Vec::<Word>::deserialize(reader)
                    .map_err(|_| ProgramError::InvalidInstructionData)?;
                let entrypoint_index =
                    u32::from_le_bytes(entrypoint_index_bytes.try_into().unwrap());
                let card_index = u32::from_le_bytes(card_index_bytes.try_into().unwrap());

                Ok(Self::ProcessAction {
                    card_index,
                    entrypoint_index,
                    args,
                })
            } else {
                Err(ProgramError::InvalidInstructionData)
            }
        } else {
            Err(ProgramError::InvalidInstructionData)
        }
    }
}
