//! Solcery DB internal structures
#![deny(unsafe_op_in_unsafe_fn)]
mod raw;

use account_fs::{FSAllocator, FSDispatcher};
use raw::column::Column;
use raw::index::Index;

pub struct DBManager<'a> {
    fs: &mut FSDispatcher<'a>,
}
