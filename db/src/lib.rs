//! Solcery DB internal structures
#![deny(unsafe_op_in_unsafe_fn)]

use bytemuck::{cast_mut, cast_slice_mut};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::mem;
use std::rc::Rc;
use tinyvec::SliceVec;

mod column;
mod raw;

use account_fs::{SegmentId, FS};
use column::Column;
use raw::column::ColumnHeader;
use raw::enums::{ColumnType, Data, DataType};
use raw::index::Index;

type FSCell<'a> = Rc<RefCell<FS<'a>>>;

pub struct DB<'a> {
    fs: FSCell<'a>,
    index: &'a mut Index,
    column_headers: SliceVec<'a, ColumnHeader>,
    accessed_columns: BTreeMap<String, Box<dyn Column>>,
}

impl<'a> DB<'a> {
    pub fn from_segment(fs: FSCell<'a>, segment: SegmentId) -> Result<Self, ()> {
        let db_segment = fs.borrow_mut().segment(segment).unwrap(); //TODO: Error handing

        let (index, columns): (&'a mut [u8], &'a mut [u8]) =
            db_segment.split_at_mut(mem::size_of::<Index>());

        let index: &mut [[u8; mem::size_of::<Index>()]] = cast_slice_mut(index);
        let index: &mut Index = cast_mut(&mut index[0]);

        assert!(index.check_magic());

        let columns: &mut [ColumnHeader] = cast_slice_mut(columns);

        let mut column_headers = SliceVec::from_slice_len(columns, index.column_count());

        Ok(Self {
            fs,
            index,
            column_headers,
            accessed_columns: BTreeMap::new(),
        })
    }

    pub fn init_in_segment(fs: FSCell<'a>, max_colums: usize, max_rows: usize) -> Result<Self, ()> {
        unimplemented!();
    }

    pub fn add_column(&mut self, name: &str, dtype: DataType, is_secondary_key: bool) {
        unimplemented!();
    }

    pub fn remove_column(&mut self, name: &str) {
        unimplemented!();
    }

    pub fn value(&self, primary_key: DataType, column: u32) -> DataType {
        unimplemented!();
    }

    pub fn value_secondary(
        &self,
        key_column: &str,
        secondary_key: DataType,
        column: &str,
    ) -> DataType {
        unimplemented!();
    }

    pub fn set_value(&self, primary_key: DataType, column: u32, value: DataType) {
        unimplemented!();
    }

    pub fn set_value_secondary(
        &self,
        key_column: u32,
        secondary_key: DataType,
        column: u32,
        value: DataType,
    ) {
        unimplemented!();
    }
    pub fn set_row(&self, row: Vec<DataType>) {
        unimplemented!();
    }

    pub fn row(&self, primary_key: DataType) -> Vec<DataType> {
        unimplemented!();
    }

    pub fn row_secondary_key(&self, key_column: u32, secondary_key: DataType) -> Vec<DataType> {
        unimplemented!();
    }
}
