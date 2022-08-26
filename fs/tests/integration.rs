use pretty_assertions::assert_eq;
use solana_program::pubkey::Pubkey;

use fs_test::*;

#[test]
fn full_initialization() {
    let program_id = Pubkey::new_unique();

    let mut fs = prepare_fs(&program_id);

    let segment_id = fs.allocate_segment(150).unwrap();
    let segment_id1 = fs.allocate_segment(123).unwrap();
    let segment_id2 = fs.allocate_segment(126).unwrap();
    fs.deallocate_segment(&segment_id1).unwrap();
    let segment_id3 = fs.allocate_segment(121).unwrap();

    let segment = fs.segment(&segment_id).unwrap();

    assert_eq!(segment, vec![0; 150]);
}

// TODO: I have to create a fully used FS with allocated and deallocated segments, serialize it to
// bytes and then use it.

const pubkey_0: &str = "1111111ogCyDbaRMvkdsHB3qfdyFYaG1WtRUAfdh";
const account_0: &str = "536f6c636572795f44425f4163636f756e745f4865616465720005000a000000040000000000000000960000000000000000960000010f00000003010000010f";
const pubkey_1: &str = "11111112D1oxKts8YPdTJRG5FzxTNpMtWmq8hkVx3";
const pubkey_2: &str = "11111112cMQwSC9qirWGjZM6gLGwW69X22mqwLLGP";
const pubkey_3: &str = "111111131h1vYVSYuKP6AhS86fbRdMw9XHiZAvAaj";
//const pubkey_0: Pubkey = Pubkey::new(&bs58::decode().into_vec().unwrap());
//const pubkey_1: Pubkey = Pubkey::new(bs58::decode().into_vec());
//const pubkey_2: Pubkey = Pubkey::new(bs58::decode("").into_vec());
//const pubkey_3: Pubkey = Pubkey::new(bs58::decode("").into_vec());
