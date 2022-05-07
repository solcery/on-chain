use super::*;
use pretty_assertions::assert_eq;

#[test]
fn segments() {
    let mut account_vec = vec![0; AccountAllocator::account_size(10, 1000)];

    let slice = &mut account_vec;
    let mut alloc = unsafe { AccountAllocator::init_account(slice, 10).unwrap() };

    let id_0 = alloc.allocate_chunk(10).unwrap();
    let id_1 = alloc.allocate_chunk(100).unwrap();
    let id_2 = alloc.allocate_chunk(132).unwrap();
    let id_3 = alloc.allocate_chunk(12).unwrap();
    let id_4 = alloc.allocate_chunk(15).unwrap();
    let id_5 = alloc.allocate_chunk(17).unwrap();

    alloc.deallocate_chunk(id_1).unwrap();
    let id_6 = alloc.allocate_chunk(50).unwrap();
    let id_7 = alloc.allocate_chunk(50).unwrap();

    alloc.deallocate_chunk(id_2).unwrap();
    let id_8 = alloc.allocate_chunk(40).unwrap();

    let mut data_allocator = alloc.to_data_allocator();

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
        DataError::NoSuchSegment
    );
    assert_eq!(
        data_allocator.segment(id_2).unwrap_err(),
        DataError::NoSuchSegment
    );

    assert_eq!(
        data_allocator.segment(id_0).unwrap_err(),
        DataError::AlreadyBorrowed
    );
    assert_eq!(
        data_allocator.segment(id_3).unwrap_err(),
        DataError::AlreadyBorrowed
    );
}
