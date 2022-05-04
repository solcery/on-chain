use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable)]
pub struct Column {
    name: [u8; 64],
    value_type: u8,
    account_pubkey: [u8; 32],
    segment_id: [u8; 4],
    column_type: u8, // I'm sure, that we'll never invent more than 256 table types
}

#[repr(u8)]
#[derive(Clone, Copy)]
pub enum ColumnType {
    RBTree,
    // This types are not implemented yet
    //OneToOne,
    //OneToMany,
    //ManyToOne,
    //ManyToMany,
    //RBSet,
}
