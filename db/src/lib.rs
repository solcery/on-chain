//! Solcery DB internal structures
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_debug_implementations)]
//#![deny(missing_docs)]
#![warn(missing_docs)]

use bytemuck::{cast_mut, cast_slice_mut};
use solana_program::msg;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt;
use std::mem;
use std::rc::Rc;
use tinyvec::SliceVec;

use account_fs::{SegmentId, FS};
use slice_rbtree::tree::{tree_size, TreeParams};
use solcery_reltab::one_to_one::one_to_one_size;

mod column;
mod data;
mod error;
mod params;
mod raw;

use column::Column;
use data::{from_column_slice, init_column_slice};
use raw::column::ColumnHeader;
use raw::index::Index;

pub use data::*;
pub use error::Error;
pub use params::{ColumnParams, ColumnType};
pub use raw::column_id::ColumnId;

type FSCell<'long, 'short> = Rc<RefCell<FS<'long, 'short>>>;

/// The main database structure
pub struct DB<'long: 'short, 'short> {
    fs: FSCell<'long, 'short>,
    index: &'short mut Index,
    column_headers: SliceVec<'short, ColumnHeader>,
    accessed_columns: RefCell<BTreeMap<ColumnId, Box<dyn Column + 'short>>>,
    segment: SegmentId,
}

impl<'long: 'short, 'short> DB<'long, 'short> {
    /// Constructs [`DB`] struct, assuming that the DB header is placed in the `segment`
    pub fn from_segment(fs: FSCell<'long, 'short>, segment: SegmentId) -> Result<Self, Error> {
        let db_segment = fs.borrow_mut().segment(&segment)?;

        if db_segment.len() < mem::size_of::<Index>() {
            return Err(Error::WrongSegment);
        }

        let (index, columns): (&'short mut [u8], &'short mut [u8]) =
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
            accessed_columns: RefCell::new(BTreeMap::new()),
            segment,
        })
    }

    /// Initializes [`DB`] with the given parameters in the first suitable segment.
    ///
    /// On success, this function returns [`DB`] struct and [`SegmentId`] of the segment, there it
    /// was initialized in.
    pub fn init_in_segment(
        fs: FSCell<'long, 'short>,
        table_name: &str,
        max_columns: usize,
        max_rows: usize,
        primary_key_type: DataType,
    ) -> Result<(Self, SegmentId), Error> {
        let index_size = Index::size(max_columns);
        let mut borrowed_fs = fs.borrow_mut();
        let segment = borrowed_fs.allocate_segment(index_size)?;

        // We've just successfully allocated this segment, so this operation is infailible;
        let index_slice = borrowed_fs.segment(&segment).unwrap();

        drop(borrowed_fs);

        let (index, columns): (&'short mut [u8], &'short mut [u8]) =
            index_slice.split_at_mut(mem::size_of::<Index>());

        let index: &mut [[u8; mem::size_of::<Index>()]] = cast_slice_mut(index);
        let index: &mut Index = cast_mut(&mut index[0]);

        unsafe {
            index.fill(table_name, primary_key_type, max_columns, max_rows);
        }

        let columns: &mut [ColumnHeader] = cast_slice_mut(columns);

        let column_headers = SliceVec::from_slice_len(columns, 0);

        msg!(
            "Initialized DB in segment: {} {}",
            segment.pubkey,
            segment.id
        );

        Ok((
            Self {
                fs,
                index,
                column_headers,
                accessed_columns: RefCell::new(BTreeMap::new()),
                segment,
            },
            segment,
        ))
    }

    /// Adds a new column to the [`DB`].
    ///
    /// On success, this function returns [`ColumnId`] of the added column.
    pub fn add_column(
        &mut self,
        name: &str,
        dtype: DataType,
        is_secondary_key: bool,
    ) -> Result<ColumnId, Error> {
        if self.index.column_count() == self.index.column_max() {
            return Err(Error::NoColumnsLeft);
        }

        let mut borrowed_fs = self.fs.borrow_mut();

        let size = if is_secondary_key {
            one_to_one_size(
                self.index.primary_key_type().size(),
                dtype.size(),
                self.index.max_rows(),
            )
        } else {
            tree_size(
                TreeParams {
                    k_size: self.index.primary_key_type().size(),
                    v_size: dtype.size(),
                },
                self.index.max_rows(),
            )
        };
        let segment = borrowed_fs.allocate_segment(size)?;

        // We've just successfully allocated this segment, so this operation is infailible;
        let container = borrowed_fs.segment(&segment).unwrap();
        let column = if is_secondary_key {
            init_column_slice(
                self.index.primary_key_type(),
                dtype,
                ColumnType::OneToOne,
                container,
            )
        } else {
            init_column_slice(
                self.index.primary_key_type(),
                dtype,
                ColumnType::RBTree,
                container,
            )
        }
        // init_column may only fail in case of wrong-sized slice. Here we generate the correct
        // value, so this invocation is infailible.
        .unwrap();

        drop(borrowed_fs);

        let id = ColumnId::new(self.index.generate_id());

        let column_header = if is_secondary_key {
            unsafe { ColumnHeader::new(name, id, segment, dtype, ColumnType::OneToOne) }
        } else {
            unsafe { ColumnHeader::new(name, id, segment, dtype, ColumnType::RBTree) }
        };

        self.column_headers.push(column_header);

        unsafe {
            self.index.set_column_count(self.column_headers.len());
        }

        self.accessed_columns.borrow_mut().insert(id, column);
        Ok(id)
    }

    /// Removes column from the [`DB`]
    pub fn remove_column(&mut self, column_id: ColumnId) -> Result<(), Error> {
        let (index, segment_id) = self
            .column_headers
            .iter()
            .enumerate()
            .find(|(_, &column)| column.id() == column_id)
            .map(|(index, col)| (index, col.segment_id()))
            .ok_or(Error::NoSuchColumn)?;

        let mut fs = self.fs.borrow_mut();

        if let Some(_) = self.accessed_columns.borrow_mut().remove(&column_id) {
            unsafe {
                // # Safety
                // The only mutable borrow of this segment was storred in `accessed_columns` and
                // we've just removed it,
                fs.release_borrowed_segment(&segment_id);
            }
        }
        fs.deallocate_segment(&segment_id)?;

        self.column_headers.remove(index);

        unsafe {
            self.index.set_column_count(self.column_headers.len());
        }

        Ok(())
    }

    /// Gets value in the `column_id` by its `primary_key`.
    pub fn value(&self, primary_key: Data, column_id: ColumnId) -> Result<Option<Data>, Error> {
        let mut accessed_columns = self.accessed_columns.borrow_mut();

        if let Some(column) = accessed_columns.get(&column_id) {
            Ok(column.get_value(primary_key))
        } else {
            let column_header = self
                .column_headers
                .iter()
                .find(|&col| col.id() == column_id)
                .ok_or(Error::NoSuchColumn)?;

            let column_slice = self.fs.borrow_mut().segment(&column_header.segment_id())?;

            let column = from_column_slice(
                self.index.primary_key_type(),
                column_header.value_type(),
                column_header.column_type(),
                column_slice,
            )?;

            let value = column.get_value(primary_key);

            accessed_columns.insert(column_header.id(), column);

            Ok(value)
        }
    }

    /// Gets value in the `column_id` by its `secondary_key`, located in `key_column_id`.
    pub fn value_secondary(
        &self,
        key_column_id: ColumnId,
        secondary_key: Data,
        column_id: ColumnId,
    ) -> Result<Option<Data>, Error> {
        let primary_key = self.get_primary_key(key_column_id, secondary_key)?;

        match primary_key {
            Some(key) => self.value(key, column_id),
            None => Ok(None),
        }
    }

    /// Sets `primary_key - value` pair in the `column_id`.
    pub fn set_value(
        &mut self,
        primary_key: Data,
        column_id: ColumnId,
        value: Data,
    ) -> Result<Option<Data>, Error> {
        let mut accessed_columns = self.accessed_columns.borrow_mut();

        if let Some(column) = accessed_columns.get_mut(&column_id) {
            column.set(primary_key, value)
        } else {
            let column_header = self
                .column_headers
                .iter()
                .find(|&col| col.id() == column_id)
                .ok_or(Error::NoSuchColumn)?;

            let column_slice = self.fs.borrow_mut().segment(&column_header.segment_id())?;

            let mut column = from_column_slice(
                self.index.primary_key_type(),
                column_header.value_type(),
                column_header.column_type(),
                column_slice,
            )?;

            let result = column.set(primary_key, value);

            accessed_columns.insert(column_header.id(), column);

            result
        }
    }

    /// Sets `primary_key - value` pair in the `column_id`, where `primary_key` is optained from `secondary_key` in `key_column_id`.
    pub fn set_value_secondary(
        &mut self,
        key_column_id: ColumnId,
        secondary_key: Data,
        column_id: ColumnId,
        value: Data,
    ) -> Result<Option<Data>, Error> {
        let primary_key = self.get_primary_key(key_column_id, secondary_key)?;

        match primary_key {
            Some(key) => self.set_value(key, column_id, value),
            None => Err(Error::SecondaryKeyWithNonExistentPrimaryKey),
        }
    }

    /// Deletes `primary_key - value` pair in the `column_id`.
    pub fn delete_value(&mut self, primary_key: Data, column_id: ColumnId) -> Result<bool, Error> {
        let mut accessed_columns = self.accessed_columns.borrow_mut();

        if let Some(column) = accessed_columns.get_mut(&column_id) {
            Ok(column.delete_by_key(primary_key))
        } else {
            let column_header = self
                .column_headers
                .iter()
                .find(|&col| col.id() == column_id)
                .ok_or(Error::NoSuchColumn)?;

            let column_slice = self.fs.borrow_mut().segment(&column_header.segment_id())?;

            let mut column = from_column_slice(
                self.index.primary_key_type(),
                column_header.value_type(),
                column_header.column_type(),
                column_slice,
            )?;

            let was_value_present = column.delete_by_key(primary_key);

            accessed_columns.insert(column_header.id(), column);

            Ok(was_value_present)
        }
    }

    /// Deletes `primary_key - value` pair in the `column_id`, where `primary_key` is optained from `secondary_key` in `key_column_id`.
    pub fn delete_value_secondary(
        &mut self,
        key_column_id: ColumnId,
        secondary_key: Data,
        column_id: ColumnId,
    ) -> Result<bool, Error> {
        let primary_key = self.get_primary_key(key_column_id, secondary_key)?;

        match primary_key {
            Some(key) => self.delete_value(key, column_id),
            None => Err(Error::SecondaryKeyWithNonExistentPrimaryKey),
        }
    }

    /// Sets values in each column with the given `primary_key`.
    ///
    /// Returns `true` if there were any old values in the row, otherwise returns `false`.
    pub fn set_row<Row>(&mut self, primary_key: Data, row: Row) -> Result<bool, Error>
    where
        Row: IntoIterator<Item = (ColumnId, Data)>,
    {
        //FIXME: this opretion is not atomic
        row.into_iter()
            .map(|(column, value)| {
                self.set_value(primary_key.clone(), column, value)
                    .map(|old_val| old_val.is_some())
            })
            .reduce(|acc, result| match (acc, result) {
                (Ok(sum), Ok(value)) => Ok(sum || value),
                (Err(err), _) => Err(err),
                (Ok(_), Err(err)) => Err(err),
            })
            //FIXME: we should report empty rows as errors
            .expect("Row should be non-empty")
    }

    /// Gets [`BTreeMap`] of `column <-> value` for a given `primary_key`.
    pub fn row(&self, primary_key: Data) -> Result<BTreeMap<ColumnId, Option<Data>>, Error> {
        let mut accessed_columns = self.accessed_columns.borrow_mut();
        self.column_headers
            .iter()
            .map(|column_header| {
                let column_id = column_header.id();

                if let Some(column) = accessed_columns.get(&column_id) {
                    Ok((column_id, column.get_value(primary_key.clone())))
                } else {
                    let column_slice = self.fs.borrow_mut().segment(&column_header.segment_id())?;

                    let column = from_column_slice(
                        self.index.primary_key_type(),
                        column_header.value_type(),
                        column_header.column_type(),
                        column_slice,
                    )?;

                    let value = column.get_value(primary_key.clone());

                    accessed_columns.insert(column_header.id(), column);

                    Ok((column_id, value))
                }
            })
            .collect()
    }

    /// Gets [`BTreeMap`] of `column <-> value`, there `primary_key` is derived from the
    /// `secondary_key` in the `key_column_id`.
    pub fn row_secondary_key(
        &self,
        key_column_id: ColumnId,
        secondary_key: Data,
    ) -> Result<BTreeMap<ColumnId, Option<Data>>, Error> {
        let primary_key = self.get_primary_key(key_column_id, secondary_key)?;

        match primary_key {
            Some(key) => self.row(key),
            None => Err(Error::SecondaryKeyWithNonExistentPrimaryKey),
        }
    }

    /// Deletes all values, assosiated with the wiven `primary_key`, will return `Err(_)` if not
    /// all columns are accessible.
    pub fn delete_row(&mut self, primary_key: Data) -> Result<(), Error> {
        let fs = self.fs.borrow();
        let mut columns = Vec::with_capacity(self.column_headers.len());
        for &header in self.column_headers.iter() {
            if !fs.is_accessible(&header.segment_id()) {
                return Err(Error::NotAllColumnsArePresent);
            }
            columns.push(header.id());
        }

        drop(fs);

        for id in columns {
            self.delete_value(primary_key.clone(), id)?;
        }

        Ok(())
    }

    /// Deletes all values, assosiated with the wiven `secondary_key`, will return `Err(_)` if not
    /// all columns are accessible.
    pub fn delete_row_secondary(
        &mut self,
        key_column_id: ColumnId,
        secondary_key: Data,
    ) -> Result<(), Error> {
        let primary_key = self.get_primary_key(key_column_id, secondary_key)?;

        match primary_key {
            Some(key) => self.delete_row(key),
            None => Err(Error::SecondaryKeyWithNonExistentPrimaryKey),
        }
    }

    /// Colmpeletely deletes [`DB`] by deallocating all the used [segments](SegmentId)
    pub fn drop_db(self) -> Result<(), Error> {
        let mut fs = self.fs.borrow_mut();

        self.accessed_columns.borrow_mut().clear();

        for &header in self.column_headers.iter() {
            if !fs.is_accessible(&header.segment_id()) {
                return Err(Error::NotAllColumnsArePresent);
            }
        }

        for &header in self.column_headers.iter() {
            let segment_id = header.segment_id();
            unsafe {
                // # Safety
                // All the borrows of the segments where placed inside the `accessed_columns`,
                // which we've just dropped, so there are no dangling pointers
                fs.release_borrowed_segment(&segment_id);
                fs.deallocate_segment(&segment_id)?;
            }
        }

        unsafe {
            // # Safety
            // The header segment of the DB was splitted into two parts: `index` and `column_header`
            // Both will be dropped, so there are no dangling pointers
            fs.release_borrowed_segment(&self.segment);
            fs.deallocate_segment(&self.segment)?;
        }

        Ok(())
    }

    fn get_primary_key(
        &self,
        key_column_id: ColumnId,
        secondary_key: Data,
    ) -> Result<Option<Data>, Error> {
        let mut accessed_columns = self.accessed_columns.borrow_mut();

        if let Some(key_column) = accessed_columns.get(&key_column_id) {
            Ok(key_column.get_key(secondary_key))
        } else {
            let column_header = self
                .column_headers
                .iter()
                .find(|&col| col.id() == key_column_id)
                .ok_or(Error::NoSuchColumn)?;

            let column_slice = self.fs.borrow_mut().segment(&column_header.segment_id())?;

            let key_column = from_column_slice(
                self.index.primary_key_type(),
                column_header.value_type(),
                column_header.column_type(),
                column_slice,
            )?;

            let primary_key = key_column.get_key(secondary_key);

            accessed_columns.insert(column_header.id(), key_column);

            Ok(primary_key)
        }
    }
}

impl<'long, 'short> Drop for DB<'long, 'short> {
    fn drop(&mut self) {
        let DB {
            fs,
            index: _,
            column_headers,
            accessed_columns,
            segment,
        } = self;

        let mut fs = fs.borrow_mut();

        for &column_id in accessed_columns.borrow().keys() {
            let column_header = column_headers
                .iter()
                .find(|&col| col.id() == column_id)
                .unwrap();

            let column_segment = column_header.segment_id();
            unsafe {
                // # Safety
                // drop will remove all references to borrowed segments, so there will be no
                // dangling pointers
                fs.release_borrowed_segment(&column_segment);
            }
        }

        unsafe {
            // # Safety
            // The header segment of the DB was splitted into two parts: `index` and `column_header`
            // Both will be dropped, so there are no dangling pointers
            fs.release_borrowed_segment(segment);
        }
    }
}

impl<'long, 'short> fmt::Debug for DB<'long, 'short> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut accessed_columns = match self.accessed_columns.try_borrow_mut() {
            Ok(x) => x,
            Err(e) => {
                return f.write_fmt(format_args!("DB is in use: {e}"));
            }
        };

        let fs = match self.fs.try_borrow() {
            Ok(x) => x,
            Err(e) => {
                return f.write_fmt(format_args!("FS is in use: {e}"));
            }
        };

        for &header in self.column_headers.iter() {
            if !accessed_columns.contains_key(&header.id())
                && fs.is_accessible(&header.segment_id())
            {
                let column_slice = self
                    .fs
                    .borrow_mut()
                    .segment(&header.segment_id())
                    .expect("Failed to borrow segment for Debug print");

                let column = from_column_slice(
                    self.index.primary_key_type(),
                    header.value_type(),
                    header.column_type(),
                    column_slice,
                )
                .expect("Failed to create column from column_slice");

                accessed_columns.insert(header.id(), column);
            }
        }

        f.debug_struct("DB")
            .field("index", &self.index)
            .field("column_headers", &self.column_headers)
            .field("columns", &accessed_columns)
            .finish()
    }
}
