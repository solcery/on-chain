use solana_program::pubkey::Pubkey;

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SegmentId {
    pub pubkey: Pubkey,
    pub id: u32,
}
