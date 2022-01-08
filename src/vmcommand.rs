use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryFrom;
use std::convert::TryInto;

#[derive(Copy, Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum VMCommand {
    /// Halts the virtual machine.
    /// This is the only way to stop the VM.
    Halt,

    // Arithmetic
    /// Returns `x + y`, where `x` is the first value from the stack add `y` is the second
    Add,
    /// Returns `x - y`, where `x` is the first value from the stack add `y` is the second
    Sub,
    /// Returns `x / y`, where `x` is the first value from the stack add `y` is the second
    Div,
    /// Returns `x * y`, where `x` is the first value from the stack add `y` is the second
    Mul,
    /// Returns `x % y`, where `x` is the first value from the stack add `y` is the second
    Rem,
    /// Negates the topmost value on the stack
    Neg,
    /// Increments the topmost value on the stack
    Inc,
    /// Decrements the topmost value on the stack
    Dec,
    /// Returns the absolute value of the topmost value from the stack
    Abs,

    // Logic
    /// Returns `x == y`, where `x` is the first value from the stack add `y` is the second
    Eq,
    /// Returns `x > y`, where `x` is the first value from the stack add `y` is the second
    Gt,
    /// Returns `x < y`, where `x` is the first value from the stack add `y` is the second
    Lt,
    /// Returns `x && y`, where `x` is the first value from the stack add `y` is the second
    And,
    /// Returns `x || y`, where `x` is the first value from the stack add `y` is the second
    Or,
    /// Logically negates the topmost value on the stack
    Not,

    // Data transfer
    /// Pushes external value on the stack
    PushConstant(Word),
    /// Reads player input from the InputTape
    ReadPlayerInput,
    /// Reads random input from the RandomTape
    ReadRandomInput,
    LoadRegionAttr {
        region_index: u16,
        attr_index: u16,
    },
    StoreRegionAttr {
        region_index: u16,
        attr_index: u16,
    },
    LoadLocal {
        index: u32,
    },
    StoreLocal {
        index: u32,
    },
    LoadArgument {
        index: u32,
    },
    StoreArgument {
        index: u32,
    },

    // Flow control
    Goto(u32),
    IfGoto(u32),
    Function {
        n_locals: u32,
    },
    Call {
        address: u32, // Actually, it is u24, but there are no such type
        n_args: u8,
    },
    Return,
    /// For functions that does not return a value
    ReturnVoid,

    // Interactions with cards
    /// Pushes total number of cards to the stack
    PushCardCount {
        region_index: u32,
    },
    /// Pushes total number of card types to the stack
    PushTypeCount,
    /// Pushes [CardType](crate::card::CardType) of the `i`-th card in the region, where `i` is the topmost value on the stack
    PushCardTypeByCardIndex {
        region_index: u32,
    },
    /// Pushes [CardType](crate::card::CardType) of the card with `id = i`, where `i` is the topmost value on the stack
    PushCardTypeByCardId,
    /// Pushes total number of cards with [CardType](crate::card::CardType) popped from the stack
    PushCardCountWithCardType {
        region_index: u32,
    },
    /// Pushes `attr_index`-th attribute of the [CardType](crate::card::CardType), those index
    /// among [CardTypes](crate::card::CardType) is on the top of the stack
    LoadCardTypeAttrByTypeIndex {
        attr_index: u32,
    },
    /// Pushes `attr_index`-th attribute of the [CardType](crate::card::CardType) of the card,
    /// those index is on the top of the stack
    LoadCardTypeAttrByCardIndex {
        region_index: u16,
        attr_index: u16,
    },
    /// Pushes `attr_index`-th attribute of the [CardType](crate::card::CardType) of the card,
    /// those id is on the top of the stack
    LoadCardTypeAttrByCardId {
        attr_index: u32,
    },
    /// Pushes `attr_index`-th attribute of the [Card](crate::card::Card),
    /// those index is on the top of the stack
    LoadCardAttrByCardIndex {
        region_index: u16,
        attr_index: u16,
    },
    /// Pops `attr_index`-th attribute of the [Card](crate::card::Card),
    /// those index is on the top of the stack
    StoreCardAttrByCardIndex {
        region_index: u16,
        attr_index: u16,
    },
    /// Pushes `attr_index`-th attribute of the [Card](crate::card::Card),
    /// those id is on the top of the stack
    LoadCardAttrByCardId {
        attr_index: u32,
    },
    /// Pops `attr_index`-th attribute of the [Card](crate::card::Card),
    /// those id is on the top of the stack
    StoreCardAttrByCardId {
        attr_index: u32,
    },

    /// Adds Card with [CardType](crate::card::CardType) index popped from the stack
    InstanceCardByTypeIndex {
        region_index: u16,
    },
    /// Adds Card with [CardType](crate::card::CardType) id popped from the stack
    InstanceCardByTypeId {
        region_index: u16,
    },
    /// CardType index and action index should be placed on the stack
    CallCardAction,
    RemoveCardByIndex {
        region_index: u16,
    },
    RemoveCardById,
}

impl Default for VMCommand {
    fn default() -> Self {
        Self::Halt
    }
}

pub type CommandByteCode = [u8; 5];

impl TryFrom<CommandByteCode> for VMCommand {
    //TODO: implement a dedicated error type
    type Error = &'static str;
    fn try_from(word: CommandByteCode) -> Result<Self, Self::Error> {
        let (discriminant, args) = word.split_at(1);
        match discriminant[0] {
            0 => Ok(Self::Halt),
            1 => Ok(Self::Add),
            2 => Ok(Self::Sub),
            3 => Ok(Self::Div),
            4 => Ok(Self::Mul),
            5 => Ok(Self::Rem),
            6 => Ok(Self::Neg),
            7 => Ok(Self::Inc),
            8 => Ok(Self::Dec),
            9 => Ok(Self::Abs),
            10 => Ok(Self::Eq),
            11 => Ok(Self::Gt),
            12 => Ok(Self::Lt),
            13 => Ok(Self::And),
            14 => Ok(Self::Or),
            15 => Ok(Self::Not),
            16 => Ok(Self::PushConstant(Word::Numeric(i32::from_le_bytes(
                args.try_into().unwrap(),
            )))),
            17 => {
                let bool_data = args[0];
                match bool_data {
                    0 => Ok(Self::PushConstant(Word::Boolean(false))),
                    1 => Ok(Self::PushConstant(Word::Boolean(true))),
                    _ => Err("PushConstant argument corrupted."),
                }
            }
            18 => Ok(Self::ReadPlayerInput),
            19 => Ok(Self::ReadRandomInput),
            20 => {
                let (region, attr) = args.split_at(2);
                Ok(Self::LoadRegionAttr {
                    region_index: u16::from_le_bytes(region.try_into().unwrap()),
                    attr_index: u16::from_le_bytes(attr.try_into().unwrap()),
                })
            }
            21 => {
                let (region, attr) = args.split_at(2);
                Ok(Self::StoreRegionAttr {
                    region_index: u16::from_le_bytes(region.try_into().unwrap()),
                    attr_index: u16::from_le_bytes(attr.try_into().unwrap()),
                })
            }
            22 => Ok(Self::LoadLocal {
                index: u32::from_le_bytes(args.try_into().unwrap()),
            }),
            23 => Ok(Self::StoreLocal {
                index: u32::from_le_bytes(args.try_into().unwrap()),
            }),
            24 => Ok(Self::LoadArgument {
                index: u32::from_le_bytes(args.try_into().unwrap()),
            }),
            25 => Ok(Self::StoreArgument {
                index: u32::from_le_bytes(args.try_into().unwrap()),
            }),

            26 => Ok(Self::Goto(u32::from_le_bytes(args.try_into().unwrap()))),
            27 => Ok(Self::IfGoto(u32::from_le_bytes(args.try_into().unwrap()))),
            28 => Ok(Self::Function {
                n_locals: u32::from_le_bytes(args.try_into().unwrap()),
            }),

            29 => {
                // Actually, addresses are 24 bit wide, so the maximum number of instructions is
                // 2^16 (approx. 81MB total, so it is bigger than the maximum account size)
                let mut address_bytes = [0; 4];
                address_bytes[..3].clone_from_slice(&word[1..4]);

                Ok(Self::Call {
                    address: u32::from_le_bytes(address_bytes),

                    n_args: word[4],
                })
            }
            30 => Ok(Self::Return),
            31 => Ok(Self::ReturnVoid),
            32 => Ok(Self::PushCardCount {
                region_index: u32::from_le_bytes(args.try_into().unwrap()),
            }),
            33 => Ok(Self::PushTypeCount),
            34 => Ok(Self::PushCardTypeByCardIndex {
                region_index: u32::from_le_bytes(args.try_into().unwrap()),
            }),
            35 => Ok(Self::PushCardTypeByCardId),
            36 => Ok(Self::PushCardCountWithCardType {
                region_index: u32::from_le_bytes(args.try_into().unwrap()),
            }),
            37 => Ok(Self::LoadCardTypeAttrByTypeIndex {
                attr_index: u32::from_le_bytes(args.try_into().unwrap()),
            }),
            38 => {
                let (region, attr) = args.split_at(2);
                Ok(Self::LoadCardTypeAttrByCardIndex {
                    region_index: u16::from_le_bytes(region.try_into().unwrap()),
                    attr_index: u16::from_le_bytes(attr.try_into().unwrap()),
                })
            }
            39 => Ok(Self::LoadCardTypeAttrByCardId {
                attr_index: u32::from_le_bytes(args.try_into().unwrap()),
            }),
            40 => {
                let (region, attr) = args.split_at(2);
                Ok(Self::LoadCardAttrByCardIndex {
                    region_index: u16::from_le_bytes(region.try_into().unwrap()),
                    attr_index: u16::from_le_bytes(attr.try_into().unwrap()),
                })
            }
            41 => {
                let (region, attr) = args.split_at(2);
                Ok(Self::StoreCardAttrByCardIndex {
                    region_index: u16::from_le_bytes(region.try_into().unwrap()),
                    attr_index: u16::from_le_bytes(attr.try_into().unwrap()),
                })
            }
            42 => Ok(Self::LoadCardAttrByCardId {
                attr_index: u32::from_le_bytes(args.try_into().unwrap()),
            }),
            43 => Ok(Self::StoreCardAttrByCardId {
                attr_index: u32::from_le_bytes(args.try_into().unwrap()),
            }),
            44 => Ok(Self::InstanceCardByTypeIndex {
                region_index: u16::from_le_bytes(args[0..2].try_into().unwrap()),
            }),
            45 => Ok(Self::InstanceCardByTypeId {
                region_index: u16::from_le_bytes(args[0..2].try_into().unwrap()),
            }),
            46 => Ok(Self::CallCardAction),
            47 => Ok(Self::RemoveCardByIndex {
                region_index: u16::from_le_bytes(args[0..2].try_into().unwrap()),
            }),
            48 => Ok(Self::RemoveCardById),
            _ => Err("Illegal instruction"),
        }
    }
}

impl From<VMCommand> for CommandByteCode {
    fn from(instruction: VMCommand) -> Self {
        match instruction {
            VMCommand::Halt => [0, 0, 0, 0, 0],
            VMCommand::Add => [1, 0, 0, 0, 0],
            VMCommand::Sub => [2, 0, 0, 0, 0],
            VMCommand::Div => [3, 0, 0, 0, 0],
            VMCommand::Mul => [4, 0, 0, 0, 0],
            VMCommand::Rem => [5, 0, 0, 0, 0],
            VMCommand::Neg => [6, 0, 0, 0, 0],
            VMCommand::Inc => [7, 0, 0, 0, 0],
            VMCommand::Dec => [8, 0, 0, 0, 0],
            VMCommand::Abs => [9, 0, 0, 0, 0],
            VMCommand::Eq => [10, 0, 0, 0, 0],
            VMCommand::Gt => [11, 0, 0, 0, 0],
            VMCommand::Lt => [12, 0, 0, 0, 0],
            VMCommand::And => [13, 0, 0, 0, 0],
            VMCommand::Or => [14, 0, 0, 0, 0],
            VMCommand::Not => [15, 0, 0, 0, 0],
            VMCommand::PushConstant(word) => match word {
                Word::Numeric(val) => {
                    let val_bytes = val.to_le_bytes();
                    let mut byte_code = [16, 0, 0, 0, 0];
                    byte_code[1..].copy_from_slice(&val_bytes);
                    byte_code
                }
                Word::Boolean(false) => [17, 0, 0, 0, 0],
                Word::Boolean(true) => [17, 1, 0, 0, 0],
            },
            VMCommand::ReadPlayerInput => [18, 0, 0, 0, 0],
            VMCommand::ReadRandomInput => [19, 0, 0, 0, 0],
            VMCommand::LoadRegionAttr {
                region_index,
                attr_index,
            } => {
                let region_index_bytes = region_index.to_le_bytes();
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [20, 0, 0, 0, 0];
                byte_code[1..3].copy_from_slice(&region_index_bytes);
                byte_code[3..].copy_from_slice(&attr_index_bytes);
                byte_code
            }
            VMCommand::StoreRegionAttr {
                region_index,
                attr_index,
            } => {
                let region_index_bytes = region_index.to_le_bytes();
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [21, 0, 0, 0, 0];
                byte_code[1..3].copy_from_slice(&region_index_bytes);
                byte_code[3..].copy_from_slice(&attr_index_bytes);
                byte_code
            }
            VMCommand::LoadLocal { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [22, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                byte_code
            }
            VMCommand::StoreLocal { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [23, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                byte_code
            }
            VMCommand::LoadArgument { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [24, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                byte_code
            }
            VMCommand::StoreArgument { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [25, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                byte_code
            }
            VMCommand::Goto(address) => {
                let address_bytes = address.to_le_bytes();
                let mut byte_code = [26, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&address_bytes);
                byte_code
            }
            VMCommand::IfGoto(address) => {
                let address_bytes = address.to_le_bytes();
                let mut byte_code = [27, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&address_bytes);
                byte_code
            }
            VMCommand::Function { n_locals } => {
                let n_locals_bytes = n_locals.to_le_bytes();
                let mut byte_code = [28, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&n_locals_bytes);
                byte_code
            }
            VMCommand::Call { address, n_args } => {
                let address_bytes = address.to_le_bytes(); // [u8;4]
                let mut byte_code: [u8; 5] = [29, 0, 0, 0, 0];
                byte_code[1..4].copy_from_slice(&address_bytes[0..3]);
                byte_code[4] = n_args as u8;
                byte_code
            }
            VMCommand::Return => [30, 0, 0, 0, 0],
            VMCommand::ReturnVoid => [31, 0, 0, 0, 0],
            VMCommand::PushCardCount { region_index } => {
                let region_index_bytes = region_index.to_le_bytes();
                let mut byte_code: [u8; 5] = [32, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&region_index_bytes);
                byte_code
            }
            VMCommand::PushTypeCount => [33, 0, 0, 0, 0],
            VMCommand::PushCardTypeByCardIndex { region_index } => {
                let region_index_bytes = region_index.to_le_bytes();
                let mut byte_code: [u8; 5] = [34, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&region_index_bytes);
                byte_code
            }
            VMCommand::PushCardTypeByCardId => [35, 0, 0, 0, 0],
            VMCommand::PushCardCountWithCardType { region_index } => {
                let region_index_bytes = region_index.to_le_bytes();
                let mut byte_code: [u8; 5] = [36, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&region_index_bytes);
                byte_code
            }
            VMCommand::LoadCardTypeAttrByTypeIndex { attr_index } => {
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [37, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&attr_index_bytes);
                byte_code
            }
            VMCommand::LoadCardTypeAttrByCardIndex {
                region_index,
                attr_index,
            } => {
                let region_index_bytes = region_index.to_le_bytes();
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [38, 0, 0, 0, 0];
                byte_code[1..3].copy_from_slice(&region_index_bytes);
                byte_code[3..].copy_from_slice(&attr_index_bytes);
                byte_code
            }
            VMCommand::LoadCardTypeAttrByCardId { attr_index } => {
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [39, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&attr_index_bytes);
                byte_code
            }
            VMCommand::LoadCardAttrByCardIndex {
                region_index,
                attr_index,
            } => {
                let region_index_bytes = region_index.to_le_bytes();
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [40, 0, 0, 0, 0];
                byte_code[1..3].copy_from_slice(&region_index_bytes);
                byte_code[3..].copy_from_slice(&attr_index_bytes);
                byte_code
            }
            VMCommand::StoreCardAttrByCardIndex {
                region_index,
                attr_index,
            } => {
                let region_index_bytes = region_index.to_le_bytes();
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [41, 0, 0, 0, 0];
                byte_code[1..3].copy_from_slice(&region_index_bytes);
                byte_code[3..].copy_from_slice(&attr_index_bytes);
                byte_code
            }
            VMCommand::LoadCardAttrByCardId { attr_index } => {
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [42, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&attr_index_bytes);
                byte_code
            }
            VMCommand::StoreCardAttrByCardId { attr_index } => {
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [43, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&attr_index_bytes);
                byte_code
            }
            VMCommand::InstanceCardByTypeIndex { region_index } => {
                let region_index_bytes = region_index.to_le_bytes();
                let mut byte_code = [44, 0, 0, 0, 0];
                byte_code[1..3].copy_from_slice(&region_index_bytes);
                byte_code
            }
            VMCommand::InstanceCardByTypeId { region_index } => {
                let region_index_bytes = region_index.to_le_bytes();
                let mut byte_code = [45, 0, 0, 0, 0];
                byte_code[1..3].copy_from_slice(&region_index_bytes);
                byte_code
            }
            VMCommand::CallCardAction => [46, 0, 0, 0, 0],
            VMCommand::RemoveCardByIndex { region_index } => {
                let region_index_bytes = region_index.to_le_bytes();
                let mut byte_code = [47, 0, 0, 0, 0];
                byte_code[1..3].copy_from_slice(&region_index_bytes);
                byte_code
            }
            VMCommand::RemoveCardById => [48, 0, 0, 0, 0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(VMCommand::Halt)]
    #[test_case(VMCommand::Add)]
    #[test_case(VMCommand::Sub)]
    #[test_case(VMCommand::Mul)]
    #[test_case(VMCommand::Div)]
    #[test_case(VMCommand::Rem)]
    #[test_case(VMCommand::Neg)]
    #[test_case(VMCommand::Inc)]
    #[test_case(VMCommand::Dec)]
    #[test_case(VMCommand::Abs)]
    #[test_case(VMCommand::Eq)]
    #[test_case(VMCommand::Gt)]
    #[test_case(VMCommand::Lt)]
    #[test_case(VMCommand::Or)]
    #[test_case(VMCommand::And)]
    #[test_case(VMCommand::Not)]
    #[test_case(VMCommand::PushConstant(Word::Boolean(true)))]
    #[test_case(VMCommand::PushConstant(Word::Boolean(false)))]
    #[test_case(VMCommand::PushConstant(Word::Numeric(123)))]
    #[test_case(VMCommand::PushConstant(Word::Numeric(-124)))]
    #[test_case(VMCommand::ReadPlayerInput)]
    #[test_case(VMCommand::ReadRandomInput)]
    #[test_case(VMCommand::LoadRegionAttr{region_index: 100, attr_index: 7})]
    #[test_case(VMCommand::StoreRegionAttr{region_index: 101, attr_index: 9})]
    #[test_case(VMCommand::LoadLocal{index: 124})]
    #[test_case(VMCommand::StoreLocal{index: 125})]
    #[test_case(VMCommand::LoadArgument{index: 113})]
    #[test_case(VMCommand::StoreArgument{index: 114})]
    #[test_case(VMCommand::Goto(126))]
    #[test_case(VMCommand::IfGoto(120))]
    #[test_case(VMCommand::Function { n_locals: 102 })]
    #[test_case(VMCommand::Call { address: 2, n_args: 103 })]
    #[test_case(VMCommand::Return)]
    #[test_case(VMCommand::ReturnVoid)]
    #[test_case(VMCommand::PushCardCount{region_index: 104})]
    #[test_case(VMCommand::PushTypeCount)]
    #[test_case(VMCommand::PushCardTypeByCardIndex { region_index: 45 })]
    #[test_case(VMCommand::PushCardTypeByCardId)]
    #[test_case(VMCommand::PushCardCountWithCardType{ region_index: 46 })]
    #[test_case(VMCommand::LoadCardTypeAttrByTypeIndex { attr_index: 47 })]
    #[test_case(VMCommand::LoadCardTypeAttrByCardIndex { region_index: 5, attr_index: 3 })]
    #[test_case(VMCommand::LoadCardTypeAttrByCardId { attr_index: 105 })]
    #[test_case(VMCommand::LoadCardAttrByCardIndex { region_index: 11, attr_index: 106 })]
    #[test_case(VMCommand::StoreCardAttrByCardIndex { region_index: 12, attr_index: 107 })]
    #[test_case(VMCommand::LoadCardAttrByCardId { attr_index: 108 })]
    #[test_case(VMCommand::StoreCardAttrByCardId { attr_index: 109 })]
    #[test_case(VMCommand::InstanceCardByTypeIndex{ region_index: 110 })]
    #[test_case(VMCommand::InstanceCardByTypeId{ region_index: 111 })]
    #[test_case(VMCommand::CallCardAction)]
    #[test_case(VMCommand::RemoveCardByIndex {region_index: 13})]
    #[test_case(VMCommand::RemoveCardById)]
    fn bytecode_to_instruction_equivalence(instruction: VMCommand) {
        let bytecode = CommandByteCode::from(instruction);
        let decoded_instruction = VMCommand::try_from(bytecode).unwrap();
        pretty_assertions::assert_eq!(instruction, decoded_instruction);
    }
}
