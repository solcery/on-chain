use pretty_assertions::assert_eq;
use solana_program::pubkey::Pubkey;

use account_fs::FS;
use fs_test::*;

#[test]
fn full_initialization() {
    let program_id = Pubkey::new_unique();

    let account_params = AccountParams {
        address: None,
        owner: program_id.clone(),
        data: AccountData::Empty(10_000),
    };
    let mut fs_data = FSAccounts::replicate_params(account_params, 3);

    let account_infos = fs_data.account_info_iter();
    let mut fs = FS::from_uninit_account_iter(&program_id, &mut account_infos.iter(), 10).unwrap();

    let segment_id = fs.allocate_segment(150).unwrap();

    let segment = fs.segment(&segment_id).unwrap();

    assert_eq!(segment, vec![0; 150]);
}
