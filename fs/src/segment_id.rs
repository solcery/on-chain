use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshDeserialize, BorshSerialize, Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct SegmentId {
    pub pubkey: Pubkey,
    pub id: u32,
}
