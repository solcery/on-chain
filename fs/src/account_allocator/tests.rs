use super::*;
use pretty_assertions::assert_eq;

#[test]
fn account_initialization() {
    let mut account_vec = vec![0; AccountAllocator::account_size(10, 1000)];

    let alloc = { AccountAllocator::init_account(&mut account_vec, 10).unwrap() };

    assert_eq!(alloc.allocation_table.inodes_count(), 1);
    assert_eq!(alloc.allocation_table.inodes_max(), 10);
    assert_eq!(alloc.len, 1000);

    drop(alloc);

    let alloc = { AccountAllocator::from_account(&mut account_vec).unwrap() };

    assert_eq!(alloc.allocation_table.inodes_count(), 1);
    assert_eq!(alloc.allocation_table.inodes_max(), 10);
    assert_eq!(alloc.len, 1000);
    assert_eq!(alloc.inode_data.len(), 1);
    assert_eq!(alloc.inode_data[0], {
        Inode::from_raw_parts(0, 1000, None)
    });
}

#[test]
fn allocation() {
    let mut account_vec = vec![0; AccountAllocator::account_size(10, 1000)];

    let mut alloc = { AccountAllocator::init_account(&mut account_vec, 10).unwrap() };

    let id = alloc.allocate_segment(10).unwrap();

    assert_eq!(alloc.allocation_table.inodes_count(), 2);
    assert_eq!(id, 0);
    assert_eq!(alloc.inode_data[0], {
        Inode::from_raw_parts(0, 10, Some(0))
    });
    assert_eq!(alloc.inode_data[1], {
        Inode::from_raw_parts(10, 1000, None)
    });
}

#[test]
fn deallocation() {
    let mut account_vec = vec![0; AccountAllocator::account_size(10, 1000)];

    let mut alloc = { AccountAllocator::init_account(&mut account_vec, 10).unwrap() };

    let id = alloc.allocate_segment(10).unwrap();

    alloc.deallocate_segment(id).unwrap();

    assert_eq!(alloc.allocation_table.inodes_count(), 1);
    assert_eq!(alloc.inode_data[0], {
        Inode::from_raw_parts(0, 1000, None)
    });
}

#[test]
fn segments() {
    let mut account_vec = vec![0; AccountAllocator::account_size(10, 1000)];

    let slice = &mut account_vec;
    let mut alloc = { AccountAllocator::init_account(slice, 10).unwrap() };

    let id_0 = alloc.allocate_segment(10).unwrap();
    let id_1 = alloc.allocate_segment(100).unwrap();
    let id_2 = alloc.allocate_segment(132).unwrap();
    let id_3 = alloc.allocate_segment(12).unwrap();
    let id_4 = alloc.allocate_segment(15).unwrap();
    let id_5 = alloc.allocate_segment(17).unwrap();

    alloc.deallocate_segment(id_1).unwrap();
    let id_6 = alloc.allocate_segment(50).unwrap();
    let id_7 = alloc.allocate_segment(50).unwrap();

    alloc.deallocate_segment(id_2).unwrap();
    let id_8 = alloc.allocate_segment(40).unwrap();

    let mut data_allocator = alloc;

    let slice_0 = data_allocator.segment(id_0).unwrap();
    let slice_3 = data_allocator.segment(id_3).unwrap();
    let slice_4 = data_allocator.segment(id_4).unwrap();
    let slice_5 = data_allocator.segment(id_5).unwrap();
    let slice_6 = data_allocator.segment(id_6).unwrap();
    let slice_7 = data_allocator.segment(id_7).unwrap();
    let slice_8 = data_allocator.segment(id_8).unwrap();

    assert_eq!(slice_0.len(), 10);
    assert_eq!(slice_3.len(), 12);
    assert_eq!(slice_4.len(), 15);
    assert_eq!(slice_5.len(), 17);
    assert_eq!(slice_6.len(), 50);
    assert_eq!(slice_7.len(), 50);
    assert_eq!(slice_8.len(), 40);

    assert_eq!(
        data_allocator.segment(id_1).unwrap_err(),
        Error::NoSuchSegment
    );
    assert_eq!(
        data_allocator.segment(id_2).unwrap_err(),
        Error::NoSuchSegment
    );

    assert_eq!(
        data_allocator.segment(id_0).unwrap_err(),
        Error::AlreadyBorrowed
    );
    assert_eq!(
        data_allocator.segment(id_3).unwrap_err(),
        Error::AlreadyBorrowed
    );
}

#[test]
fn merge() {
    let mut account_vec = vec![0; AccountAllocator::account_size(10, 210)];

    let slice = &mut account_vec;
    let mut alloc = { AccountAllocator::init_account(slice, 10).unwrap() };

    let id_0 = alloc.allocate_segment(50).unwrap();
    let id_1 = alloc.allocate_segment(50).unwrap();
    alloc.allocate_segment(100).unwrap();

    alloc.deallocate_segment(id_0).unwrap();
    alloc.deallocate_segment(id_1).unwrap();

    let err = alloc.allocate_segment(120).unwrap_err();
    assert_eq!(err, Error::NoSuitableSegmentFound);

    alloc.allocate_segment(100).unwrap();
    alloc.allocate_segment(10).unwrap();
}
