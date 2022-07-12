use account_fs::FS;
use pretty_assertions::assert_eq;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

#[derive(Clone, Debug, Eq, PartialEq)]
struct AccountParams {
    owner: Pubkey,
    data: Data,
    is_signer: bool,
    is_writable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct InternalAccountInfo {
    key: Pubkey,
    lamports: u64,
    data: Vec<u8>,
    owner: Pubkey,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Data {
    Filled(Vec<u8>),
    Empty(usize),
}

fn prepare_account_info(params: AccountParams) -> AccountInfo<'static> {
    let data = match params.data {
        Data::Filled(vec) => vec,
        Data::Empty(cap) => vec![0; cap],
    };

    let internal_info: &'static mut InternalAccountInfo =
        Box::leak(Box::new(InternalAccountInfo {
            key: Pubkey::new_unique(),
            lamports: 1,
            data,
            owner: params.owner,
        }));

    AccountInfo::new(
        &internal_info.key,
        params.is_signer,
        params.is_writable,
        &mut internal_info.lamports,
        &mut internal_info.data,
        &internal_info.owner,
        false,
        1,
    )
}

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

    let mut fs = FS::from_uninit_account_iter(&mut accounts.iter(), 10).unwrap();

    let segment_id = fs.allocate_segment(150).unwrap();

    let segment = fs.segment(segment_id).unwrap();

    assert_eq!(segment, vec![0; 150]);
}
