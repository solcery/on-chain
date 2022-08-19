use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{cast, cast_mut, cast_slice, cast_slice_mut, Pod, Zeroable};
use solana_program::{
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack, Sealed},
};

pub const MINT_SEED: &[u8] = b"DB-program_mint";
pub const GLOBAL_STATE_SEED: &[u8] = b"DB-program_global_state";
const STATE_MAGIC: &[u8; 13] = b"DBGlobalState";

#[derive(Debug, Eq, PartialEq, Zeroable, BorshDeserialize, BorshSerialize, Copy, Clone, Pod)]
#[repr(C)]
pub struct DBGlobalState {
    magic: [u8; 13],
    global_state_bump: u8,
    mint_bump: u8,
}

impl DBGlobalState {
    #[must_use]
    pub fn global_state_bump(&self) -> u8 {
        self.global_state_bump
    }

    #[must_use]
    pub fn mint_bump(&self) -> u8 {
        self.mint_bump
    }

    #[must_use]
    pub fn new(global_state_bump: u8, mint_bump: u8) -> Self {
        Self {
            magic: *STATE_MAGIC,
            global_state_bump,
            mint_bump,
        }
    }
}

impl Sealed for DBGlobalState {}

impl IsInitialized for DBGlobalState {
    fn is_initialized(&self) -> bool {
        &self.magic == STATE_MAGIC
    }
}

impl Pack for DBGlobalState {
    const LEN: usize = std::mem::size_of::<DBGlobalState>();
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let slice: &mut [[u8; Self::LEN]] = cast_slice_mut(dst);
        let state: &mut DBGlobalState = cast_mut(&mut slice[0]);
        *state = *self;
    }
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let slice: &[[u8; Self::LEN]] = cast_slice(src);
        let state: DBGlobalState = cast(slice[0]);
        Ok(state)
    }
}
