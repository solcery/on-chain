use std::collections::BTreeMap;

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct GameObject {
    uuid: Uuid,
    parent: Uuid,
    attributes: Node,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Node {
    ident: String,
    value: NodeValue,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub enum NodeValue {
    Int(i32),
    Bool(bool),
    Object(Box<GameObject>),
    ObjectArray(Vec<GameObject>),
    NodeMap(BTreeMap<String, Node>),
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Hash)]
pub struct Uuid([u8; 32]);
