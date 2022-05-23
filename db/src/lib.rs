//! Solcery DB internal structures
#![deny(unsafe_op_in_unsafe_fn)]
// Temporary added this, so the output of the compiler is not flooded with unused warnings
#![allow(unused_variables)]
#![allow(dead_code)]

use bytemuck::{cast_mut, cast_slice_mut};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::mem;
use std::rc::Rc;
use tinyvec::SliceVec;

mod column;
mod raw;

use account_fs::{FSError, SegmentId, FS};
use column::Column;
use raw::column::ColumnHeader;
use raw::index::Index;
use solcery_data_types::db::schema::{Data, DataType};

type FSCell<'a> = Rc<RefCell<FS<'a>>>;

pub struct DB<'a> {
    fs: FSCell<'a>,
    index: &'a mut Index,
    column_headers: SliceVec<'a, ColumnHeader>,
    accessed_columns: BTreeMap<u32, Box<dyn Column>>,
    segment: SegmentId,
}

impl<'a> DB<'a> {
    pub fn from_segment(fs: FSCell<'a>, segment: SegmentId) -> Result<Self, Error> {
        let db_segment = fs.borrow_mut().segment(segment).unwrap(); //TODO: Error handing

        let (index, columns): (&'a mut [u8], &'a mut [u8]) =
            db_segment.split_at_mut(mem::size_of::<Index>());

        let index: &mut [[u8; mem::size_of::<Index>()]] = cast_slice_mut(index);
        let index: &mut Index = cast_mut(&mut index[0]);

        assert!(index.check_magic());

        let columns: &mut [ColumnHeader] = cast_slice_mut(columns);

        let column_headers = SliceVec::from_slice_len(columns, index.column_count());

        Ok(Self {
            fs,
            index,
            column_headers,
            accessed_columns: BTreeMap::new(),
            segment,
        })
    }

    pub fn init_in_segment(
        fs: FSCell<'a>,
        table_name: &str,
        max_columns: usize,
        max_rows: usize,
        primary_key_type: DataType,
    ) -> Result<Self, Error> {
        let index_size = Index::size(max_columns);
        let mut borrowed_fs = fs.borrow_mut();
        let segment = borrowed_fs.allocate_segment(index_size)?;

        // We've just successfully allocated this segment, so this operation is infailible;
        let index_slice = borrowed_fs.segment(segment).unwrap();

        drop(borrowed_fs);

        let (index, columns): (&'a mut [u8], &'a mut [u8]) =
            index_slice.split_at_mut(mem::size_of::<Index>());

        let index: &mut [[u8; mem::size_of::<Index>()]] = cast_slice_mut(index);
        let index: &mut Index = cast_mut(&mut index[0]);

        unsafe {
            index.fill(table_name, primary_key_type, max_columns, max_rows);
        }

        let columns: &mut [ColumnHeader] = cast_slice_mut(columns);

        let column_headers = SliceVec::from_slice_len(columns, 0);

        Ok(Self {
            fs,
            index,
            column_headers,
            accessed_columns: BTreeMap::new(),
            segment,
        })
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

    pub fn remove_db(self) -> Result<(), ()> {
        unimplemented!();
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    FSError(FSError),
}

impl From<FSError> for Error {
    fn from(err: FSError) -> Self {
        Self::FSError(err)
    }
}
