//! # In-slice allocator for BTree
//! Data layout inside the slice is the following: [Root](Root) struct + raw nodes data
//!
//! There are two types of nodes: [InternalNode](InternalNode) and [LeafNode](LeafNode), which
//! occupy different space.
//!
//! For effectively allocating nodes inside slice we use a fusion of double-ended stack allocator
//! and pool allocator.
//!
//! Allocator state consists of stack pointer and `Option<head node pointer>` for both node types.
//!
//! ## Allocate
//! if Some(head pointer), then copy pointer from that node to new head pointer and return the
//! node else check that stack pointer + node size is less than other node's stack pointer.
//! if it is true, increment stack pointer and return the node
//! else return error
//!
//! ## Deallocate
//! if the node to deallocate is on the top of the stack, than decrement the stack pointer
//! else write the contents of head node pointer to the begining of the node and write `Some(node
//! offset)` to the head ode pointer.
use bytemuck_derive::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Copy, Clone, Zeroable)]
struct Root {
    // Allocator state
    leaf_free_ptr: u32,
    leaf_head_ptr: u32,

    internal_node_free_ptr: u32,
    internal_node_head_ptr: u32,

    node_ptr: u32, // Root node pointer
    // BTree parameters
    size: u32,
    k_size: u16,
    v_size: u16,
    height: u16,
    b: u8,
    // This is done for aligment reasons
    // match allocator_flag {
    // 0 => (leaf_head_ptr, internal_node_head_ptr) == (None, None),
    // 1 => (leaf_head_ptr, internal_node_head_ptr) == (Some(ptr), None),
    // 2 => (leaf_head_ptr, internal_node_head_ptr) == (None, Some(ptr)),
    // 3 => (leaf_head_ptr, internal_node_head_ptr) == (Some(ptr), Some(ptr)),
    // _ => CORRUPTED!!!,
    // }
    allocator_flag: u8,
}

#[repr(C)]
struct LeafNode<const K_SIZE: usize, const V_SIZE: usize, const CAPACITY: usize> {
    len: u8,
    keys: [[u8; K_SIZE]; CAPACITY],
    values: [[u8; V_SIZE]; CAPACITY],
}

#[repr(C)]
struct InternalNode<
    const K_SIZE: usize,
    const V_SIZE: usize,
    const PTR_CAP: usize,
    const CAPACITY: usize,
> {
    // PTR_CAP == 2B
    // CAPACITY == 2B-1
    pointers: [u32; PTR_CAP],
    len: u8,
    keys: [[u8; K_SIZE]; CAPACITY],
    values: [[u8; V_SIZE]; CAPACITY],
}
