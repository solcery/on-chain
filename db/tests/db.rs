use borsh::{BorshDeserialize, BorshSerialize};
use pretty_assertions::assert_eq;
use solana_program::pubkey::Pubkey;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::rc::Rc;

use account_fs::*;
use fs_test::*;
use solcery_db::*;

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
        let name = Data::ShortString(ShortString::try_from("Alice").unwrap());
        let old_val = db.set_value(id, col_id, name).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(1);
        let name = Data::ShortString(ShortString::try_from("Bob").unwrap());
        let old_val = db.set_value(id, col_id, name).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(2);
        let name = Data::ShortString(ShortString::try_from("Carol").unwrap());
        let old_val = db.set_value(id, col_id, name).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(3);
        let name = Data::ShortString(ShortString::try_from("Chad").unwrap());
        let old_val = db.set_value(id, col_id, name).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(4);
        let name = Data::ShortString(ShortString::try_from("Eve").unwrap());
        let old_val = db.set_value(id, col_id, name).unwrap();
        assert_eq!(old_val, None);
    }

    {
        // Age
        let column_name = "Age";
        let dtype = DataType::Int;
        let col_id = db.add_column(column_name, dtype, false).unwrap();

        let id = Data::Int(0);
        let name = Data::Int(22);
        let old_val = db.set_value(id, col_id, name).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(1);
        let name = Data::Int(23);
        let old_val = db.set_value(id, col_id, name).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(2);
        let name = Data::Int(22);
        let old_val = db.set_value(id, col_id, name).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(3);
        let name = Data::Int(20);
        let old_val = db.set_value(id, col_id, name).unwrap();
        assert_eq!(old_val, None);

        let id = Data::Int(4);
        let name = Data::Int(30);
        let old_val = db.set_value(id, col_id, name).unwrap();
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
    assert_eq!(
        obtained_val,
        Some(Data::ShortString(ShortString::try_from("Alice").unwrap()))
    );
    db.remove_column(name_column).unwrap();

    let age_column = ColumnId::new(1);
    db.remove_column(age_column).unwrap();

    let err = db.value(Data::Int(0), ColumnId::new(0)).unwrap_err();
    assert_eq!(err, Error::NoSuchColumn);

    let err = db.value(Data::Int(0), ColumnId::new(1)).unwrap_err();
    assert_eq!(err, Error::NoSuchColumn);
}

#[test]
fn set_value() {
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

    let old_val = db
        .set_value(
            Data::Int(5),
            name_column,
            Data::ShortString(ShortString::try_from("Alec").unwrap()),
        )
        .unwrap();
    assert_eq!(old_val, None);

    let old_val = db
        .set_value(
            Data::Int(5),
            name_column,
            Data::ShortString(ShortString::try_from("Alex").unwrap()),
        )
        .unwrap();
    assert_eq!(
        old_val,
        Some(Data::ShortString(ShortString::try_from("Alec").unwrap()))
    );

    let name_column = ColumnId::new(1);

    let old_val = db.value(Data::Int(5), name_column).unwrap();
    assert_eq!(old_val, None);

    let old_val = db
        .set_value(Data::Int(5), name_column, Data::Int(18))
        .unwrap();
    assert_eq!(old_val, None);

    let old_val = db
        .set_value(Data::Int(5), name_column, Data::Int(19))
        .unwrap();
    assert_eq!(old_val, Some(Data::Int(18)));
}

#[test]
fn set_value_secondary() {
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
    let age_column = ColumnId::new(1);

    let err = db
        .set_value_secondary(
            name_column,
            Data::ShortString(ShortString::try_from("Alex").unwrap()),
            age_column,
            Data::Int(20),
        )
        .unwrap_err();
    assert_eq!(err, Error::SecondaryKeyWithNonExistentPrimaryKey);

    let old_val = db
        .set_value_secondary(
            name_column,
            Data::ShortString(ShortString::try_from("Alice").unwrap()),
            age_column,
            Data::Int(20),
        )
        .unwrap();
    assert_eq!(old_val, Some(Data::Int(22)));
}

#[test]
fn value() {
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

    let db = DB::from_segment(fs.clone(), DB_SEGMENT).unwrap();

    let name_column = ColumnId::new(0);
    let age_column = ColumnId::new(1);

    let val = db.value(Data::Int(6), age_column).unwrap();

    assert_eq!(val, None);

    let val = db.value(Data::Int(0), age_column).unwrap();

    assert_eq!(val, Some(Data::Int(22)));

    let val = db.value(Data::Int(0), name_column).unwrap();

    assert_eq!(
        val,
        Some(Data::ShortString(ShortString::try_from("Alice").unwrap()))
    );

    let val = db.value(Data::Int(6), name_column).unwrap();

    assert_eq!(val, None);
}

#[test]
fn value_secondary() {
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

    let db = DB::from_segment(fs.clone(), DB_SEGMENT).unwrap();

    let name_column = ColumnId::new(0);
    let age_column = ColumnId::new(1);

    let val = db
        .value_secondary(
            name_column,
            Data::ShortString(ShortString::try_from("Bob").unwrap()),
            age_column,
        )
        .unwrap();

    assert_eq!(val, Some(Data::Int(23)));

    let val = db
        .value_secondary(
            name_column,
            Data::ShortString(ShortString::try_from("Alex").unwrap()),
            age_column,
        )
        .unwrap();

    assert_eq!(val, None);
}

#[test]
fn delete_value() {
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

    let age_column = ColumnId::new(1);

    let val = db.delete_value(Data::Int(6), age_column).unwrap();

    assert_eq!(val, false);

    let val = db.delete_value(Data::Int(0), age_column).unwrap();

    assert_eq!(val, true);

    let val = db.delete_value(Data::Int(0), age_column).unwrap();

    assert_eq!(val, false);
}

#[test]
fn delete_value_secondary() {
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
    let age_column = ColumnId::new(1);

    let val = db
        .delete_value_secondary(
            name_column,
            Data::ShortString(ShortString::try_from("Bob").unwrap()),
            age_column,
        )
        .unwrap();

    assert_eq!(val, true);

    let val = db
        .delete_value_secondary(
            name_column,
            Data::ShortString(ShortString::try_from("Bob").unwrap()),
            age_column,
        )
        .unwrap();

    assert_eq!(val, false);
}

#[test]
fn set_row() {
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
    let age_column = ColumnId::new(1);

    let val = db.value(Data::Int(6), age_column).unwrap();

    assert_eq!(val, None);

    let new_row = vec![
        (
            ColumnId::new(0),
            Data::ShortString(ShortString::try_from("Ann").unwrap()),
        ),
        (ColumnId::new(1), Data::Int(29)),
    ];

    let new_row = BTreeMap::from_iter(new_row.into_iter());

    db.set_row(Data::Int(6), new_row.clone()).unwrap();

    let added_row = db.row(Data::Int(6)).unwrap();

    let added_row =
        BTreeMap::from_iter(added_row.into_iter().filter_map(|(k, v)| v.map(|v| (k, v))));

    assert_eq!(added_row, new_row);

    let added_row = db
        .row_secondary_key(
            name_column,
            Data::ShortString(ShortString::try_from("Ann").unwrap()),
        )
        .unwrap();

    let added_row =
        BTreeMap::from_iter(added_row.into_iter().filter_map(|(k, v)| v.map(|v| (k, v))));

    assert_eq!(added_row, new_row);
}

#[test]
fn delete_row() {
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

    db.delete_row(Data::Int(6)).unwrap();
    db.delete_row(Data::Int(0)).unwrap();

    let row: Vec<_> = db.row(Data::Int(0)).unwrap().into_values().collect();

    assert_eq!(row, vec![None, None]);
}

#[test]
fn delete_row_secondary() {
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

    let err = db
        .delete_row_secondary(
            name_column,
            Data::ShortString(ShortString::try_from("Ann").unwrap()),
        )
        .unwrap_err();
    db.delete_row_secondary(
        name_column,
        Data::ShortString(ShortString::try_from("Alice").unwrap()),
    )
    .unwrap();

    let row: Vec<_> = db.row(Data::Int(0)).unwrap().into_values().collect();

    assert_eq!(err, Error::SecondaryKeyWithNonExistentPrimaryKey);
    assert_eq!(row, vec![None, None]);
}

#[test]
fn drop_db() {
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

    let db = DB::from_segment(fs.clone(), DB_SEGMENT).unwrap();

    db.drop_db().unwrap();
}

// This function was used to create an image of empty FS, which is now used as a basis for DB
// creation
#[cfg_attr(tarpaulin, ignore)]
fn _fs_initialization() {
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

    let filename = format!("{}/tests/fs_images/clean_fs", env!("CARGO_MANIFEST_DIR"));

    let mut file = File::create(filename).unwrap();

    fs_data.serialize(&mut file).unwrap();
}
