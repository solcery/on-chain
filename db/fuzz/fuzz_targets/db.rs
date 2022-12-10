#![no_main]

use libfuzzer_sys::arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use solana_program::pubkey::Pubkey;
use std::cell::RefCell;
use std::rc::Rc;

use account_fs::{SegmentId, FS};
use fs_test::*;

fuzz_target!(|data: FuzzHarness| {
    // fuzzed code goes here
    let FuzzHarness { params, methods } = harness;
    let Some(first_param) = params.get(0) else {
        return;
    };
    let program_id = Pubkey::new_from_array(first_param.owner);
    let len = params.len();
    let len = if len > 3 { len - 2 } else { len };

    let mut fs_data = FSAccounts::from_params_iter(params.clone().into_iter());
    let pubkeys: Vec<_> = fs_data.0.iter().map(InternalAccountInfo::key).collect();
    let account_infos = fs_data.account_info_iter();

    let Ok(mut fs) =
        FS::from_uninit_account_iter(&program_id, &mut account_infos.iter().take(len), 10) else {return;};

    let fs = Rc::new(RefCell::new(fs));

    for method in methods {
        use DBMethod::*;
        match method {
            Init {
                name,
                max_columns,
                max_rows,
                primary_key_type,
            } => {
                return;
            }
        }
    }
});

#[derive(Debug, Arbitrary)]
struct FuzzHarness {
    params: Vec<AccountParams>,
    methods: Vec<DBMethod>,
}

#[derive(Debug, Arbitrary)]
enum DBMethod {
    Init {
        name: String,
        max_columns: u8,
        max_rows: u16,
        primary_key_type: DataType,
    },
    //AllocateSegment { size: usize },
    //DeallocateSegment { id: u32, pubkey_id: usize },
    //Segment { id: u32, pubkey_id: usize },
    //ReleaseBorrowedSegment { id: u32, pubkey_id: usize },
}

fn derive_segment_id(params: &Vec<Pubkey>, id: u32, pubkey_id: usize) -> Option<SegmentId> {
    if pubkey_id < params.len() {
        Some(SegmentId {
            id,
            pubkey: params[pubkey_id],
        })
    } else {
        None
    }
}
