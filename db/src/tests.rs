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
    let (db, segment) = DB::init_in_segment(
        fs.clone(),
        table_name,
        max_columns,
        max_rows,
        primary_key_type,
    )
    .unwrap();

    drop(db);

    DB::from_segment(fs.clone(), segment).unwrap();
}

#[test]
fn columns() {
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

    let column_name = "Test Column";
    let dtype = DataType::Int;
    let col_id = db.add_column(column_name, dtype, false).unwrap();

    drop(db);

    let mut db = DB::from_segment(fs.clone(), segment).unwrap();

    db.remove_column(col_id).unwrap();
}

#[test]
fn values() {
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
