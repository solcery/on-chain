//! Solcery DB internal structures
#![deny(unsafe_op_in_unsafe_fn)]
// Temporary added this, so the output of the compiler is not flooded with unused warnings
#![allow(unused_variables)]
#![allow(dead_code)]

use bytemuck::{cast_mut, cast_slice_mut};
use solana_program::pubkey::Pubkey as PK;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::mem;
use std::rc::Rc;
use tinyvec::SliceVec;

use account_fs::{FSError, SegmentId, FS};
use slice_rbtree::{tree_size, Error as RBTreeError, RBTree};
use solcery_data_types::db::schema::{ColumnType, Data, DataType};

mod column;
mod raw;

use column::Column;
use raw::column::ColumnHeader;
use raw::index::Index;

type FSCell<'a> = Rc<RefCell<FS<'a>>>;

pub struct DB<'a> {
    fs: FSCell<'a>,
    index: &'a mut Index,
    column_headers: SliceVec<'a, ColumnHeader>,
    accessed_columns: BTreeMap<u32, Box<dyn Column + 'a>>,
    segment: SegmentId,
}

impl<'a> DB<'a> {
    pub fn from_segment(fs: FSCell<'a>, segment: SegmentId) -> Result<Self, Error> {
        let db_segment = fs.borrow_mut().segment(segment)?;

        if db_segment.len() < mem::size_of::<Index>() {
            return Err(Error::WrongSegment);
        }

        let (index, columns): (&'a mut [u8], &'a mut [u8]) =
            db_segment.split_at_mut(mem::size_of::<Index>());

        let index: &mut [[u8; mem::size_of::<Index>()]] = cast_slice_mut(index);
        let index: &mut Index = cast_mut(&mut index[0]);

        if !index.check_magic() {
            return Err(Error::WrongSegment);
        }

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

    pub fn add_column(
        &mut self,
        name: &str,
        dtype: DataType,
        is_secondary_key: bool,
    ) -> Result<u32, Error> {
        if self.index.column_count() == self.index.column_max() {
            return Err(Error::NoColumnsLeft);
        }

        let mut borrowed_fs = self.fs.borrow_mut();
        let segment = borrowed_fs.allocate_segment(tree_size(
            self.index.primary_key_type().size(),
            dtype.size(),
            self.index.max_rows(),
        ))?;

        // We've just successfully allocated this segment, so this operation is infailible;
        let tree = borrowed_fs.segment(segment).unwrap();
        let column =
            Self::init_slice(self.index.primary_key_type(), dtype, is_secondary_key, tree).unwrap();
        drop(borrowed_fs);

        let id = self.index.generate_id();
        let column_header =
            unsafe { ColumnHeader::new(name, id, segment, dtype, ColumnType::RBTree) };

        self.column_headers.push(column_header);

        unsafe {
            self.index.set_column_count(self.column_headers.len());
        }

        self.accessed_columns.insert(id, column);
        Ok(id)
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

    fn init_slice<'b: 'a>(
        pk_type: DataType,
        val_type: DataType,
        is_secondary_key: bool,
        slice: &'b mut [u8],
    ) -> Result<Box<dyn Column + 'a>, Error> {
        use DataType::*;
        if is_secondary_key {
            unimplemented!();
        } else {
            match (pk_type, val_type) {
                //TODO: I should implement a procedural macro for that
                // Int
                (Int, Int) => RBTree::<u32, u32, 4, 4>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (Int, Pubkey) => RBTree::<u32, PK, 4, 64>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (Int, ShortString) => RBTree::<u32, String, 4, 16>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (Int, MediumString) => RBTree::<u32, String, 4, 64>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (Int, LongString) => RBTree::<u32, String, 4, 256>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                // Pubkey
                (Pubkey, Int) => RBTree::<PK, u32, 64, 4>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (Pubkey, Pubkey) => RBTree::<PK, PK, 64, 64>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (Pubkey, ShortString) => RBTree::<PK, String, 64, 16>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (Pubkey, MediumString) => RBTree::<PK, String, 64, 64>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (Pubkey, LongString) => RBTree::<PK, String, 64, 256>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                // ShortString
                (ShortString, Int) => RBTree::<String, u32, 16, 4>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (ShortString, Pubkey) => RBTree::<String, PK, 16, 64>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (ShortString, ShortString) => RBTree::<String, String, 16, 16>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (ShortString, MediumString) => RBTree::<String, String, 16, 64>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (ShortString, LongString) => RBTree::<String, String, 16, 256>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                // MediumString
                (MediumString, Int) => RBTree::<String, u32, 64, 4>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (MediumString, Pubkey) => RBTree::<String, PK, 64, 64>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (MediumString, ShortString) => RBTree::<String, String, 64, 16>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (MediumString, MediumString) => RBTree::<String, String, 64, 64>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (MediumString, LongString) => RBTree::<String, String, 64, 256>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                // LongString
                (LongString, Int) => RBTree::<String, u32, 256, 4>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (LongString, Pubkey) => RBTree::<String, PK, 256, 64>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (LongString, ShortString) => RBTree::<String, String, 256, 16>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (LongString, MediumString) => RBTree::<String, String, 256, 64>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
                (LongString, LongString) => RBTree::<String, String, 256, 256>::init_slice(slice)
                    .map(|tree| Box::new(tree) as Box<dyn Column>)
                    .map_err(|e| Error::from(e)),
            }
        }
    }

    fn from_slice<'b: 'a>(
        pk_type: DataType,
        val_type: DataType,
        is_secondary_key: bool,
        slice: &'b mut [u8],
    ) -> Result<Box<dyn Column + 'a>, Error> {
        use DataType::*;
        if is_secondary_key {
            unimplemented!();
        } else {
            unsafe {
                match (pk_type, val_type) {
                    // Int
                    (Int, Int) => RBTree::<u32, u32, 4, 4>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    (Int, Pubkey) => RBTree::<u32, PK, 4, 64>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    (Int, ShortString) => RBTree::<u32, String, 4, 16>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    (Int, MediumString) => RBTree::<u32, String, 4, 64>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    (Int, LongString) => RBTree::<u32, String, 4, 256>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    // Pubkey
                    (Pubkey, Int) => RBTree::<PK, u32, 64, 4>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    (Pubkey, Pubkey) => RBTree::<PK, PK, 64, 64>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    (Pubkey, ShortString) => RBTree::<PK, String, 64, 16>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    (Pubkey, MediumString) => RBTree::<PK, String, 64, 64>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    (Pubkey, LongString) => RBTree::<PK, String, 64, 256>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    // ShortString
                    (ShortString, Int) => RBTree::<String, u32, 16, 4>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    (ShortString, Pubkey) => RBTree::<String, PK, 16, 64>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    (ShortString, ShortString) => {
                        RBTree::<String, String, 16, 16>::from_slice(slice)
                            .map(|tree| Box::new(tree) as Box<dyn Column>)
                            .map_err(|e| Error::from(e))
                    }
                    (ShortString, MediumString) => {
                        RBTree::<String, String, 16, 64>::from_slice(slice)
                            .map(|tree| Box::new(tree) as Box<dyn Column>)
                            .map_err(|e| Error::from(e))
                    }
                    (ShortString, LongString) => {
                        RBTree::<String, String, 16, 256>::from_slice(slice)
                            .map(|tree| Box::new(tree) as Box<dyn Column>)
                            .map_err(|e| Error::from(e))
                    }
                    // MediumString
                    (MediumString, Int) => RBTree::<String, u32, 64, 4>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    (MediumString, Pubkey) => RBTree::<String, PK, 64, 64>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    (MediumString, ShortString) => {
                        RBTree::<String, String, 64, 16>::from_slice(slice)
                            .map(|tree| Box::new(tree) as Box<dyn Column>)
                            .map_err(|e| Error::from(e))
                    }
                    (MediumString, MediumString) => {
                        RBTree::<String, String, 64, 64>::from_slice(slice)
                            .map(|tree| Box::new(tree) as Box<dyn Column>)
                            .map_err(|e| Error::from(e))
                    }
                    (MediumString, LongString) => {
                        RBTree::<String, String, 64, 256>::from_slice(slice)
                            .map(|tree| Box::new(tree) as Box<dyn Column>)
                            .map_err(|e| Error::from(e))
                    }
                    // LongString
                    (LongString, Int) => RBTree::<String, u32, 256, 4>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    (LongString, Pubkey) => RBTree::<String, PK, 256, 64>::from_slice(slice)
                        .map(|tree| Box::new(tree) as Box<dyn Column>)
                        .map_err(|e| Error::from(e)),
                    (LongString, ShortString) => {
                        RBTree::<String, String, 256, 16>::from_slice(slice)
                            .map(|tree| Box::new(tree) as Box<dyn Column>)
                            .map_err(|e| Error::from(e))
                    }
                    (LongString, MediumString) => {
                        RBTree::<String, String, 256, 64>::from_slice(slice)
                            .map(|tree| Box::new(tree) as Box<dyn Column>)
                            .map_err(|e| Error::from(e))
                    }
                    (LongString, LongString) => {
                        RBTree::<String, String, 256, 256>::from_slice(slice)
                            .map(|tree| Box::new(tree) as Box<dyn Column>)
                            .map_err(|e| Error::from(e))
                    }
                }
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    FSError(FSError),
    NoColumnsLeft,
    RBTreeError(RBTreeError),
    WrongSegment,
}

impl From<FSError> for Error {
    fn from(err: FSError) -> Self {
        Self::FSError(err)
    }
}

impl From<RBTreeError> for Error {
    fn from(err: RBTreeError) -> Self {
        Self::RBTreeError(err)
    }
}
