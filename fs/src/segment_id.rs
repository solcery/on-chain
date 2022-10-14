use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

/// An adress of a segment inside FS
///
/// It is guaranteed that each segment has unique [`SegmentId`]
#[derive(BorshDeserialize, BorshSerialize, Debug, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
pub struct SegmentId {
    /// [`Pubkey`] of the account, where the data resides
    pub pubkey: Pubkey,
    /// id of the segment in that account
    pub id: u32,
}
