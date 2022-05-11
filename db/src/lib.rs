//! Solcery DB internal structures
#![deny(unsafe_op_in_unsafe_fn)]

use std::collections::BTreeMap;

mod column;
mod raw;

use account_fs::FS;
use column::Column;
use raw::column::Column as RawColumn;
use raw::enums::{ColumnType, Data, DataType};
use raw::index::Index;

pub struct DBManager<'a> {
    fs: &'a mut FS<'a>,
    databases: BTreeMap<String, DB<'a>>,
}

pub struct DB<'a> {
    index: &'a mut Index,
    columns: BTreeMap<String, &'a mut dyn Column>,
    // Do we need on-the-fly column bootstrapping?
    // It will speed up DB loading.
}
