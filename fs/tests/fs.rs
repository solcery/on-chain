use borsh::BorshDeserialize;
use pretty_assertions::assert_eq;
use solana_program::pubkey::Pubkey;
use std::fs::File;
use std::io::Read;

use account_fs::FS;
use fs_test::*;

#[test]
fn initialization() {
    let program_id = Pubkey::new_unique();

    let account_params = AccountParams {
        address: None,
        owner: program_id.to_bytes(),
        data: AccountData::Empty(1_000),
    };
    let mut fs_data = FSAccounts::replicate_params(account_params, 3);

    let account_infos = fs_data.account_info_iter();
    let fs = FS::from_uninit_account_iter(&program_id, &mut account_infos.iter(), 10).unwrap();

    drop(fs);
    drop(account_infos);

    let filename = format!("{}/tests/fs_images/new_fs", env!("CARGO_MANIFEST_DIR"));

    let mut file = File::open(filename).unwrap();

    let mut clean_fs_bytes = Vec::new();
    file.read_to_end(&mut clean_fs_bytes).unwrap();

    let expected_fs_data = FSAccounts::deserialize(&mut clean_fs_bytes.as_slice()).unwrap();

    assert_eq!(fs_data, expected_fs_data);
}

#[test]
fn usage() {
    let filename = format!("{}/tests/fs_images/new_fs", env!("CARGO_MANIFEST_DIR"));

    let mut file = File::open(filename).unwrap();

    let mut clean_fs_bytes = Vec::new();
    file.read_to_end(&mut clean_fs_bytes).unwrap();

    let mut fs_data = FSAccounts::deserialize(&mut clean_fs_bytes.as_slice()).unwrap();

    let program_id = fs_data.owner_pubkey().unwrap();

    let account_infos = fs_data.account_info_iter();
    let mut fs = FS::from_uninit_account_iter(&program_id, &mut account_infos.iter(), 10).unwrap();

    let segment_id = fs.allocate_segment(150).unwrap();

    let segment = fs.segment(&segment_id).unwrap();

    assert_eq!(segment, vec![0; 150]);

    segment[0] = 123;
    segment[1] = 121;
    segment[2] = 125;
    segment[149] = 10;
    segment[140] = 1;

    let _seg_1 = fs.allocate_segment(100).unwrap();
    let _seg_2 = fs.allocate_segment(101).unwrap();
    let _seg_3 = fs.allocate_segment(800).unwrap();
    let seg_4 = fs.allocate_segment(31).unwrap();
    let seg_5 = fs.allocate_segment(310).unwrap();
    let seg_6 = fs.allocate_segment(145).unwrap();

    fs.deallocate_segment(&seg_6).unwrap();
    fs.deallocate_segment(&seg_4).unwrap();
    fs.deallocate_segment(&seg_5).unwrap();

    fs.allocate_segment(141).unwrap();
    fs.allocate_segment(14).unwrap();
    fs.allocate_segment(600).unwrap();

    drop(fs);
    drop(account_infos);

    let filename = format!("{}/tests/fs_images/used_fs", env!("CARGO_MANIFEST_DIR"));

    let mut file = File::open(filename).unwrap();

    let mut used_fs_bytes = Vec::new();
    file.read_to_end(&mut used_fs_bytes).unwrap();

    let expected_fs_data = FSAccounts::deserialize(&mut used_fs_bytes.as_slice()).unwrap();

    assert_eq!(fs_data, expected_fs_data);
}
