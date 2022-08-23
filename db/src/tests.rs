use super::*;
use pretty_assertions::assert_eq;
use solana_program::pubkey::Pubkey;

use fs_test::*;

#[test]
fn init_db() {
    let program_id = Pubkey::new_unique();
    let fs = Rc::new(RefCell::new(prepare_fs(&program_id)));

    let table_name = "Test DB";
    let max_columns = 12;
    let max_rows = 53;
    let primary_key_type = DataType::ShortString;
    let (mut db, segment) = DB::init_in_segment(
        fs.clone(),
        table_name,
        max_columns,
        max_rows,
        primary_key_type,
    )
    .unwrap();

    drop(db);

    let db = DB::from_segment(fs.clone(), segment).unwrap();
}
