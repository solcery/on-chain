use crate::vmcommand::{CommandByteCode, VMCommand};

#[derive(Debug, Eq, PartialEq)]
pub struct InstructionRom<'a> {
    instructions: &'a [VMCommand],
}

impl<'a> InstructionRom<'a> {
    #[must_use]
    pub fn fetch_instruction(&self, pc: usize) -> VMCommand {
        self.instructions[pc]
    }

    #[must_use]
    pub unsafe fn from_raw_parts(instructions: &'a [VMCommand]) -> Self {
        Self { instructions }
    }
}

pub enum Error {
    WrongSize,
}
