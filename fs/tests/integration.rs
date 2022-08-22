use account_fs::FS;
use pretty_assertions::assert_eq;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

use fs_test::*;

#[test]
fn full_initialization() {
    let program_id = Pubkey::new_unique();

    let params = AccountParams {
        owner: program_id,
        data: Data::Empty(1000),
        is_signer: false,
        is_writable: true,
    };

    let mut accounts = Vec::new();
    for _ in 0..3 {
        accounts.push(prepare_account_info(params.clone()));
    }

    let accounts: &'static mut [AccountInfo] = accounts.leak();

    let mut fs = FS::from_uninit_account_iter(&program_id, &mut accounts.iter(), 10).unwrap();

    let segment_id = fs.allocate_segment(150).unwrap();

    let segment = fs.segment(&segment_id).unwrap();

    assert_eq!(segment, vec![0; 150]);
}
