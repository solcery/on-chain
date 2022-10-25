use borsh::{BorshDeserialize, BorshSerialize};
use pretty_assertions::assert_eq;
use solana_program::pubkey::Pubkey;
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

use account_fs::*;
use fs_test::*;
use solcery_db::*;

#[test]
fn values() {
    let program_id = Pubkey::new_unique();

    let account_params = AccountParams {
        address: None,
        owner: program_id,
        data: AccountData::Empty(10_000),
    };
    let mut fs_data = FSAccounts::replicate_params(account_params, 3);

    let account_infos = fs_data.account_info_iter();
    let fs = Rc::new(RefCell::new(
        FS::from_uninit_account_iter(&program_id, &mut account_infos.iter(), 10).unwrap(),
    ));

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

    let column_name = "Test Column";
    let dtype = DataType::Int;
    let col_id = db.add_column(column_name, dtype, false).unwrap();

    let value = Data::Int(123);
    let value2 = Data::Int(125);
    let primary_key = Data::ShortString("test".to_string());
    let old_val = db
        .set_value(primary_key.clone(), col_id, value.clone())
        .unwrap();
    assert_eq!(old_val, None);
    let new_val = db
        .set_value(primary_key.clone(), col_id, value2.clone())
        .unwrap();
    assert_eq!(new_val, Some(value));

    drop(db);

    let mut db = DB::from_segment(fs.clone(), segment).unwrap();

    let obtained_val = db.value(primary_key.clone(), col_id).unwrap();
    assert_eq!(obtained_val, Some(value2));
    db.delete_value(primary_key.clone(), col_id).unwrap();

    let no_value = db.value(primary_key, col_id).unwrap();
    assert_eq!(no_value, None);
}

#[test]
fn secondary_key() {
    let program_id = Pubkey::new_unique();

    let account_params = AccountParams {
        address: None,
        owner: program_id,
        data: AccountData::Empty(10_000),
    };
    let mut fs_data = FSAccounts::replicate_params(account_params, 3);

    let account_infos = fs_data.account_info_iter();
    let fs = Rc::new(RefCell::new(
        FS::from_uninit_account_iter(&program_id, &mut account_infos.iter(), 10).unwrap(),
    ));

    let table_name = "Test DB";
    let max_columns = 12;
    let max_rows = 53;
    let primary_key_type = DataType::ShortString;
    let (mut db, _) = DB::init_in_segment(
        fs.clone(),
        table_name,
        max_columns,
        max_rows,
        primary_key_type,
    )
    .unwrap();

    let column_name = "Test Column";
    let dtype = DataType::Int;
    let col_id = db.add_column(column_name, dtype, false).unwrap();

    let value = Data::Int(123);
    let primary_key = Data::ShortString("test".to_string());
    let old_val = db
        .set_value(primary_key.clone(), col_id, value.clone())
        .unwrap();
    assert_eq!(old_val, None);

    let column_name = "Secondary Key Column";
    let dtype = DataType::Int;
    let key_col_id = db.add_column(column_name, dtype, true).unwrap();

    let secondary_value = Data::Int(15);
    let old_val = db
        .set_value(primary_key.clone(), key_col_id, secondary_value.clone())
        .unwrap();
    assert_eq!(old_val, None);

    let obtained_val = db
        .value_secondary(key_col_id, secondary_value.clone(), col_id)
        .unwrap();
    assert_eq!(obtained_val, Some(value));

    db.delete_value_secondary(key_col_id, secondary_value, col_id)
        .unwrap();

    let no_value = db.value(primary_key, col_id).unwrap();
    assert_eq!(no_value, None);
}

#[test]
fn creation_of_the_test_db() {
    let filename = format!("{}/tests/fs_images/clean_fs", env!("CARGO_MANIFEST_DIR"));

    let mut file = File::open(filename).unwrap();

    let mut clean_fs_bytes = Vec::new();
    file.read_to_end(&mut clean_fs_bytes).unwrap();

    let mut fs_data = FSAccounts::deserialize(&mut clean_fs_bytes.as_slice()).unwrap();

    let program_id = fs_data.owner_pubkey().unwrap();

    let account_infos = fs_data.account_info_iter();
    let fs = Rc::new(RefCell::new(
        FS::from_account_iter(&program_id, &mut account_infos.iter()).unwrap(),
    ));

    let table_name = "Test DB: people";
    let max_columns = 4;
    let max_rows = 10;
    let primary_key_type = DataType::Int;
    let (mut db, segment) = DB::init_in_segment(
        fs.clone(),
        table_name,
        max_columns,
        max_rows,
        primary_key_type,
    )
    .unwrap();

    {
        // Name
        let column_name = "Name";
        let dtype = DataType::ShortString;
        let col_id = db.add_column(column_name, dtype, true).unwrap();

        let id = Data::Int(0);
        let name = Data::ShortString("Alice".to_string());
        let old_val = db.set_value(id.clone(), col_id, name.clone()).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(1);
        let name = Data::ShortString("Bob".to_string());
        let old_val = db.set_value(id.clone(), col_id, name.clone()).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(2);
        let name = Data::ShortString("Carol".to_string());
        let old_val = db.set_value(id.clone(), col_id, name.clone()).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(3);
        let name = Data::ShortString("Chad".to_string());
        let old_val = db.set_value(id.clone(), col_id, name.clone()).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(4);
        let name = Data::ShortString("Eve".to_string());
        let old_val = db.set_value(id.clone(), col_id, name.clone()).unwrap();
        assert_eq!(old_val, None);
    }

    {
        // Age
        let column_name = "Age";
        let dtype = DataType::Int;
        let col_id = db.add_column(column_name, dtype, false).unwrap();

        let id = Data::Int(0);
        let name = Data::Int(22);
        let old_val = db.set_value(id.clone(), col_id, name.clone()).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(1);
        let name = Data::Int(23);
        let old_val = db.set_value(id.clone(), col_id, name.clone()).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(2);
        let name = Data::Int(22);
        let old_val = db.set_value(id.clone(), col_id, name.clone()).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(3);
        let name = Data::Int(20);
        let old_val = db.set_value(id.clone(), col_id, name.clone()).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(4);
        let name = Data::Int(30);
        let old_val = db.set_value(id.clone(), col_id, name.clone()).unwrap();
        assert_eq!(old_val, None);
    }

    dbg!(segment.pubkey.to_bytes());
    drop(db);
    drop(fs);
    drop(account_infos);

    let filename = format!("{}/tests/fs_images/prepared_db", env!("CARGO_MANIFEST_DIR"));

    let mut file = File::open(filename).unwrap();

    let mut db_fs_bytes = Vec::new();
    file.read_to_end(&mut db_fs_bytes).unwrap();

    let expected_fs_data = FSAccounts::deserialize(&mut db_fs_bytes.as_slice()).unwrap();

    assert_eq!(fs_data, expected_fs_data);
}

const DB_SEGMENT: SegmentId = SegmentId {
    pubkey: Pubkey::new_from_array([
        0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ]),
    id: 0,
};

#[test]
fn remove_column() {
    let filename = format!("{}/tests/fs_images/prepared_db", env!("CARGO_MANIFEST_DIR"));

    let mut file = File::open(filename).unwrap();

    let mut db_fs_bytes = Vec::new();
    file.read_to_end(&mut db_fs_bytes).unwrap();

    let mut fs_data = FSAccounts::deserialize(&mut db_fs_bytes.as_slice()).unwrap();

    let program_id = fs_data.owner_pubkey().unwrap();

    let account_infos = fs_data.account_info_iter();
    let fs = Rc::new(RefCell::new(
        FS::from_account_iter(&program_id, &mut account_infos.iter()).unwrap(),
    ));

    let mut db = DB::from_segment(fs.clone(), DB_SEGMENT).unwrap();

    let name_column = ColumnId::new(0);
    let obtained_val = db.value(Data::Int(0), name_column).unwrap();
    assert_eq!(obtained_val, Some(Data::ShortString("Alice".to_string())));
    db.remove_column(name_column).unwrap();

    let age_column = ColumnId::new(1);
    db.remove_column(age_column).unwrap();
}

// This function was used to create an image of empty FS, which is now used as a basis for DB
// creation
fn _fs_initialization() {
    let program_id = Pubkey::new_unique();

    let account_params = AccountParams {
        address: None,
        owner: program_id,
        data: AccountData::Empty(1_000),
    };
    let mut fs_data = FSAccounts::replicate_params(account_params, 3);

    let account_infos = fs_data.account_info_iter();
    let fs = FS::from_uninit_account_iter(&program_id, &mut account_infos.iter(), 10).unwrap();

    drop(fs);
    drop(account_infos);

    let filename = format!("{}/tests/fs_images/clean_fs", env!("CARGO_MANIFEST_DIR"));

    let mut file = File::create(filename).unwrap();

    fs_data.serialize(&mut file).unwrap();
}
