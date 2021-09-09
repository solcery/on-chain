use std::marker::Copy;

#[derive(Debug, Clone, Copy)]
pub struct Card {
    pub id: u32,
    pub card_type: u32,
    pub place: u32,
}

#[derive(Debug)]
pub struct CardType {
    pub id: u32,
    pub data: Vec<u8>,
}

impl CardType {}
