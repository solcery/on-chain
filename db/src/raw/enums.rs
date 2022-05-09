#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DataType {
    Int,
    Float,
    Pubkey,
    ShortString,  // 16 bytes
    MediumString, // 64 bytes
    LongString,   // 256 bytes
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColumnType {
    RBTree,
    // This types are not implemented yet
    //OneToOne,
    //OneToMany,
    //ManyToOne,
    //ManyToMany,
    //RBSet,
}
