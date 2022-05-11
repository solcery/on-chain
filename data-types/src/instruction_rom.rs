use crate::vmcommand::{CommandByteCode, VMCommand};
use bytemuck::cast_slice;
use std::convert::TryFrom;
use std::fmt;
use std::mem;

#[derive(Eq, PartialEq)]
pub struct InstructionRom<'a> {
    instructions: &'a [CommandByteCode],
}
impl<'a> fmt::Debug for InstructionRom<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_list()
            .entries(
                self.instructions
                    .iter()
                    .map(|bytecode| VMCommand::try_from(*bytecode).map_err(|_| bytecode)),
            )
            .finish()
    }
}

impl<'a> TryFrom<&'a [u8]> for InstructionRom<'a> {
    type Error = Error;
    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        if data.len() % mem::size_of::<CommandByteCode>() == 0 {
            let slice: &[CommandByteCode] = cast_slice(data);
            Ok(Self {
                instructions: slice,
            })
        } else {
            Err(Self::Error::WrongSize)
        }
    }
}

impl<'a> InstructionRom<'a> {
    #[must_use]
    pub fn fetch_instruction(&self, pc: usize) -> VMCommand {
        // TODO: Should use appropriate Error type
        VMCommand::try_from(self.instructions[pc]).unwrap()
    }

    #[must_use]
    pub unsafe fn from_raw_parts(instructions: &'a [CommandByteCode]) -> Self {
        Self { instructions }
    }

    #[must_use]
    pub fn from_vm_commands(instructions: &[VMCommand]) -> Vec<CommandByteCode> {
        instructions
            .iter()
            .map(|command| CommandByteCode::from(*command))
            .collect()
    }
}

#[derive(Debug)]
pub enum Error {
    WrongSize,
}
