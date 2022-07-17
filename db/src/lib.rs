//! Solcery DB internal structures
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_debug_implementations)]

use bytemuck::{cast_mut, cast_slice_mut};

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::mem;
use std::rc::Rc;
use tinyvec::SliceVec;

use account_fs::{FSError, SegmentId, FS};
use slice_rbtree::tree_size;
use solcery_data_types::db::schema::{from_column_slice, init_column_slice};

pub use solcery_data_types::db::{
    column::Column,
    error::Error,
    schema::{ColumnParams, ColumnType, Data, DataType},
};

mod raw;

use raw::column::ColumnHeader;
use raw::index::Index;

pub use raw::column_id::ColumnId;

type FSCell<'a> = Rc<RefCell<FS<'a>>>;

#[derive(Debug)]
pub struct DB<'a> {
    fs: FSCell<'a>,
    index: &'a mut Index,
    column_headers: SliceVec<'a, ColumnHeader>,
    accessed_columns: RefCell<BTreeMap<ColumnId, Box<dyn Column + 'a>>>,
    segment: SegmentId,
}

impl<'a> DB<'a> {
    pub fn from_segment(fs: FSCell<'a>, segment: SegmentId) -> Result<Self, Error> {
        let db_segment = fs.borrow_mut().segment(&segment)?;

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
            accessed_columns: RefCell::new(BTreeMap::new()),
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
        let index_slice = borrowed_fs.segment(&segment).unwrap();

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
            accessed_columns: RefCell::new(BTreeMap::new()),
            segment,
        })
    }

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
        let segment = borrowed_fs.allocate_segment(tree_size(
            self.index.primary_key_type().size(),
            dtype.size(),
            self.index.max_rows(),
        ))?;

        // We've just successfully allocated this segment, so this operation is infailible;
        let tree = borrowed_fs.segment(&segment).unwrap();
        let column = if is_secondary_key {
            unimplemented!();
        } else {
            init_column_slice(
                self.index.primary_key_type(),
                dtype,
                ColumnType::RBTree,
                tree,
            )
        }
        .unwrap();

        drop(borrowed_fs);

        let id = ColumnId::new(self.index.generate_id());
        let column_header =
            unsafe { ColumnHeader::new(name, id, segment, dtype, ColumnType::RBTree) };

        self.column_headers.push(column_header);

        unsafe {
            self.index.set_column_count(self.column_headers.len());
        }

        self.accessed_columns.borrow_mut().insert(id, column);
        Ok(id)
    }

    pub fn remove_column(&mut self, column_id: ColumnId) -> Result<(), Error> {
        let (index, segment_id) = self
            .column_headers
            .iter()
            .enumerate()
            .find(|(_, &column)| column.id() == column_id)
            .map(|(index, col)| (index, col.segment_id()))
            .ok_or(Error::NoSuchColumn)?;

        self.fs.borrow_mut().deallocate_segment(&segment_id)?;
        self.column_headers.remove(index);

        unsafe {
            self.index.set_column_count(self.column_headers.len());
        }
        Ok(())
    }

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

    pub fn set_value(
        &mut self,
        primary_key: Data,
        column_id: ColumnId,
        value: Data,
    ) -> Result<Option<Data>, Error> {
        let mut accessed_columns = self.accessed_columns.borrow_mut();

        if let Some(column) = accessed_columns.get_mut(&column_id) {
            Ok(column.set(primary_key, value))
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

            let old_value = column.set(primary_key, value);

            accessed_columns.insert(column_header.id(), column);

            Ok(old_value)
        }
    }

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

    pub fn set_row<Row>(&mut self, primary_key: Data, row: Row) -> Result<bool, Error>
    where
        Row: IntoIterator<Item = (ColumnId, Data)>,
    {
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
            .expect("Row should be non-empty")
    }

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

    pub fn drop_db(self) -> Result<(), Error> {
        let DB {
            fs,
            index,
            column_headers,
            accessed_columns,
            segment,
        } = self;

        let mut fs = fs.borrow_mut();

        for &header in column_headers.iter() {
            if !fs.is_accessible(&header.segment_id()) {
                return Err(Error::NotAllColumnsArePresent);
            }
        }

        drop(accessed_columns);

        for &header in column_headers.iter() {
            let segment_id = header.segment_id();
            unsafe {
                // # Safety
                // All the borrows of the segments where placed inside the `accessed_columns`,
                // which we've just dropped, so there are no dangling pointers
                fs.release_borrowed_segment(&segment_id);
                fs.deallocate_segment(&segment_id)?;
            }
        }

        drop(index);
        drop(column_headers);

        unsafe {
            // # Safety
            // The header segment of the DB was splitted into two parts: `index` and `column_header`
            // Both were dropped, so there are no dangling pointers
            fs.release_borrowed_segment(&segment);
            fs.deallocate_segment(&segment)?;
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

            let primary_key = key_column.get_value(secondary_key);

            accessed_columns.insert(column_header.id(), key_column);

            Ok(primary_key)
        }
    }
}
