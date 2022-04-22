use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable)]
pub struct Index {
}
