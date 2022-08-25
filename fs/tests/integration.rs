use pretty_assertions::assert_eq;
use solana_program::pubkey::Pubkey;

use fs_test::*;

#[test]
fn full_initialization() {
    let program_id = Pubkey::new_unique();

    let mut fs = prepare_fs(&program_id);

    let segment_id = fs.allocate_segment(150).unwrap();

    let segment = fs.segment(&segment_id).unwrap();

    assert_eq!(segment, vec![0; 150]);
}
