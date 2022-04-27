use super::*;
use pretty_assertions::assert_eq;
use std::mem;

fn account_size(inode_count: usize, data_size: usize) -> usize {
    mem::size_of::<AllocationTable>() + inode_count * mem::size_of::<Inode>() + data_size
}

#[test]
fn account_initialization() {
    let mut account_vec = vec![0; account_size(10, 1000)];

    let alloc = unsafe {
        AccountAllocator::init_account(&mut account_vec, 10, Pubkey::new_unique()).unwrap()
    };

    assert_eq!(alloc.allocation_table.inodes_count(), 1);
    assert_eq!(alloc.allocation_table.inodes_max(), 10);
    assert_eq!(alloc.data.len(), 1000);

    drop(alloc);

    let alloc =
        unsafe { AccountAllocator::from_account(&mut account_vec, Pubkey::new_unique()).unwrap() };

    assert_eq!(alloc.allocation_table.inodes_count(), 1);
    assert_eq!(alloc.allocation_table.inodes_max(), 10);
    assert_eq!(alloc.data.len(), 1000);
    assert_eq!(alloc.inode_data.len(), 1);
    assert_eq!(alloc.inode_data[0], unsafe {
        Inode::from_raw_parts(0, 1000, None)
    });
}

#[test]
fn allocation() {
    let mut account_vec = vec![0; account_size(10, 1000)];

    let mut alloc = unsafe {
        AccountAllocator::init_account(&mut account_vec, 10, Pubkey::new_unique()).unwrap()
    };

    let id = alloc.allocate_chunk(10).unwrap();

    assert_eq!(alloc.allocation_table.inodes_count(), 2);
    assert_eq!(id, 0);
    assert_eq!(alloc.inode_data[0], unsafe {
        Inode::from_raw_parts(0, 10, Some(0))
    });
    assert_eq!(alloc.inode_data[1], unsafe {
        Inode::from_raw_parts(10, 1000, None)
    });
}

#[test]
fn deallocation() {
    let mut account_vec = vec![0; account_size(10, 1000)];

    let mut alloc = unsafe {
        AccountAllocator::init_account(&mut account_vec, 10, Pubkey::new_unique()).unwrap()
    };

    let id = alloc.allocate_chunk(10).unwrap();

    alloc.deallocate_chunk(id).unwrap();

    assert_eq!(alloc.allocation_table.inodes_count(), 2);
    assert_eq!(alloc.inode_data[0], unsafe {
        Inode::from_raw_parts(0, 10, None)
    });
    assert_eq!(alloc.inode_data[1], unsafe {
        Inode::from_raw_parts(10, 1000, None)
    });
}
