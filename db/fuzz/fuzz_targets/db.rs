#![no_main]

use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use solana_program::pubkey::Pubkey;

use account_fs::{SegmentId, FS};
use fs_test::*;

fuzz_target!(|data: &[u8]| {
    // fuzzed code goes here
});
