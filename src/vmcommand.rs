use crate::word::Word;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum VMCommand {
    // The halting command
    Halt,
    // Arithmetic
    /// Adds two topmost values from the stack
    /// # Panics
    /// Panics if there are no enough elements on the stack or if one of the arguments is
    /// [Word::Boolean]
    Add,
    Sub,
    Div,
    /// Multiplies two topmost values from the stack
    /// # Panics
    /// Panics if there are no enough elements on the stack or if one of the arguments is
    /// [Word::Boolean]
    Mul,
    Rem,
    /// Negates the topmost value on the stack
    /// # Panics
    /// Panics if there are no enough elements on the stack or if  the argumentsis
    /// [Word::Boolean]
    Neg,
    /// Increments the topmost value on the stack
    /// Panics if there are no enough elements on the stack or if  the argumentsis
    /// [Word::Boolean]
    Inc,
    /// Decrements the topmost value on the stack
    /// Panics if there are no enough elements on the stack or if  the argumentsis
    /// [Word::Boolean]
    Dec,
    /// Computes the absolute value of the topmost value on the stack
    /// Panics if there are no enough elements on the stack or if  the argumentsis
    /// [Word::Boolean]
    Abs,

    // Logic
    Eq,
    Gt,
    Lt,
    And,
    Or,
    Not,

    // Data transfer
    PushConstant(Word),
    PushBoardAttr {
        index: u32,
    },
    PopBoardAttr {
        index: u32,
    },
    PushLocal {
        index: u32,
    },
    PopLocal {
        index: u32,
    },
    PushArgument {
        index: u32,
    },
    PopArgument {
        index: u32,
    },

    // Flow control
    Goto(u32),
    IfGoto(u32),
    Function {
        n_locals: u32,
    },
    Call {
        address: u32,
        n_args: u8,
    },
    Return,

    // Interactions with cards
    /// Pushes total number of cards to the stack
    PushCardCount,
    /// Pushes total number of card types to the stack
    PushTypeCount,
    /// Pushes [CardType](crate::card::CardType) on the `i`-th card, where `i` is the topmost value on the stack
    PushCardType,
    /// Pushes total number of cards with [CardType](crate::card::CardType) popped from the stack
    PushCardCountWithCardType,
    /// Pushes `attr_index`-th attribute of the [CardType](crate::card::CardType), those index
    /// among [CardTypes](crate::card::CardType) is on the top of the stack
    PushCardTypeAttrByTypeIndex {
        attr_index: u32,
    },
    /// Pushes `attr_index`-th attribute of the [CardType](crate::card::CardType) of the card,
    /// those index is on the top of the stack
    PushCardTypeAttrByCardIndex {
        attr_index: u32,
    },
    /// Pushes `attr_index`-th attribute of the [Card](crate::card::Card),
    /// those index is on the top of the stack
    PushCardAttr {
        attr_index: u32,
    },
    /// Pops `attr_index`-th attribute of the [Card](crate::card::Card),
    /// those index is on the top of the stack
    PopCardAttr {
        attr_index: u32,
    },

    /// Pops [CardType](crate::card::CardType) index from the stack and calls it's `action_id` action as a function
    InstanceCardByTypeIndex,
    /// Pops [CardType](crate::card::CardType) id from the stack and calls it's `action_id` action as a function
    InstanceCardByTypeId,
    /// Card index and action index should be placed on the stack
    CallCardAction,
    RemoveCardByIndex,
}

impl Default for VMCommand {
    fn default() -> Self {
        VMCommand::Halt
    }
}

type CommandByteCode = [u8; 5];

impl TryFrom<CommandByteCode> for VMCommand {
    type Error = &'static str;
    fn try_from(word: CommandByteCode) -> Result<Self, Self::Error> {
        let descriminant = word[0];
        match descriminant {
            0 => Ok(VMCommand::Halt),
            1 => Ok(VMCommand::Add),
            2 => Ok(VMCommand::Sub),
            3 => Ok(VMCommand::Div),
            4 => Ok(VMCommand::Mul),
            5 => Ok(VMCommand::Rem),
            6 => Ok(VMCommand::Neg),
            7 => Ok(VMCommand::Inc),
            8 => Ok(VMCommand::Dec),
            9 => Ok(VMCommand::Abs),
            10 => Ok(VMCommand::Eq),
            11 => Ok(VMCommand::Gt),
            12 => Ok(VMCommand::Lt),
            13 => Ok(VMCommand::And),
            14 => Ok(VMCommand::Or),
            15 => Ok(VMCommand::Not),
            16 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::PushConstant(Word::Numeric(i32::from_le_bytes(
                    val,
                )))),
                Err(_) => Err("PushConstant argument corrupted."),
            },
            17 => {
                let bool_data = word[1];
                match bool_data {
                    0 => Ok(VMCommand::PushConstant(Word::Boolean(false))),
                    1 => Ok(VMCommand::PushConstant(Word::Boolean(true))),
                    _ => Err("PushConstant argument corrupted."),
                }
            }
            18 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::PushBoardAttr {
                    index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("PushBoardAttr argument corrupted."),
            },
            19 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::PopBoardAttr {
                    index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("PopBoardAttr argument corrupted."),
            },
            20 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::PushLocal {
                    index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("PushLocal argument corrupted."),
            },
            21 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::PopLocal {
                    index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("PopLocal argument corrupted."),
            },
            22 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::PushArgument {
                    index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("PushArgument argument corrupted."),
            },
            23 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::PopArgument {
                    index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("PopArgument argument corrupted."),
            },
            24 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::Goto(u32::from_le_bytes(val))),
                Err(_) => Err("Goto argument corrupted."),
            },
            25 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::IfGoto(u32::from_le_bytes(val))),
                Err(_) => Err("IfGoto argument corrupted."),
            },
            26 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::Function {
                    n_locals: u32::from_le_bytes(val),
                }),
                Err(_) => Err("Function argument corrupted."),
            },
            27 => {
                // Actually, addresses are 24 bit wide, so the maximum number of instructions is
                // 2^16 (approx. 81MB total, so it is bigger than the maximum account size)
                let mut address_bytes = [0; 4];
                address_bytes[..3].clone_from_slice(&word[1..4]);

                Ok(VMCommand::Call {
                    address: u32::from_le_bytes(address_bytes),

                    n_args: word[4],
                })
            }
            28 => Ok(VMCommand::Return),
            29 => Ok(VMCommand::PushCardCount),
            30 => Ok(VMCommand::PushTypeCount),
            31 => Ok(VMCommand::PushCardType),
            32 => Ok(VMCommand::PushCardCountWithCardType),
            33 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::PushCardTypeAttrByTypeIndex {
                    attr_index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("PushCardTypeAttrByTypeIndex argument corrupted."),
            },
            34 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::PushCardTypeAttrByCardIndex {
                    attr_index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("PushCardTypeAttrByCardIndex argument corrupted."),
            },
            35 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::PushCardAttr {
                    attr_index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("PushCardAttr argument corrupted."),
            },
            36 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::PopCardAttr {
                    attr_index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("PopCardAttr argument corrupted."),
            },
            37 => Ok(VMCommand::InstanceCardByTypeIndex),
            38 => Ok(VMCommand::InstanceCardByTypeId),
            39 => Ok(VMCommand::CallCardAction),
            40 => Ok(VMCommand::RemoveCardByIndex),
            _ => Err("Illegal instruction"),
        }
    }
}

impl TryFrom<VMCommand> for CommandByteCode {
    type Error = &'static str;
    fn try_from(instruction: VMCommand) -> Result<Self, Self::Error> {
        match instruction {
            VMCommand::Halt => Ok([0, 0, 0, 0, 0]),
            VMCommand::Add => Ok([1, 0, 0, 0, 0]),
            VMCommand::Sub => Ok([2, 0, 0, 0, 0]),
            VMCommand::Div => Ok([3, 0, 0, 0, 0]),
            VMCommand::Mul => Ok([4, 0, 0, 0, 0]),
            VMCommand::Rem => Ok([5, 0, 0, 0, 0]),
            VMCommand::Neg => Ok([6, 0, 0, 0, 0]),
            VMCommand::Inc => Ok([7, 0, 0, 0, 0]),
            VMCommand::Dec => Ok([8, 0, 0, 0, 0]),
            VMCommand::Abs => Ok([9, 0, 0, 0, 0]),
            VMCommand::Eq => Ok([10, 0, 0, 0, 0]),
            VMCommand::Gt => Ok([11, 0, 0, 0, 0]),
            VMCommand::Lt => Ok([12, 0, 0, 0, 0]),
            VMCommand::And => Ok([13, 0, 0, 0, 0]),
            VMCommand::Or => Ok([14, 0, 0, 0, 0]),
            VMCommand::Not => Ok([15, 0, 0, 0, 0]),
            VMCommand::PushConstant(word) => match word {
                Word::Numeric(val) => {
                    let val_bytes = val.to_le_bytes();
                    let mut byte_code = [16, 0, 0, 0, 0];
                    byte_code[1..].copy_from_slice(&val_bytes);
                    Ok(byte_code)
                }
                Word::Boolean(false) => Ok([17, 0, 0, 0, 0]),
                Word::Boolean(true) => Ok([17, 1, 0, 0, 0]),
            },
            VMCommand::PushBoardAttr { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [18, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                Ok(byte_code)
            }
            VMCommand::PopBoardAttr { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [19, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                Ok(byte_code)
            }
            VMCommand::PushLocal { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [20, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                Ok(byte_code)
            }
            VMCommand::PopLocal { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [21, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                Ok(byte_code)
            }
            VMCommand::PushArgument { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [22, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                Ok(byte_code)
            }
            VMCommand::PopArgument { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [23, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                Ok(byte_code)
            }
            VMCommand::Goto(address) => {
                let address_bytes = address.to_le_bytes();
                let mut byte_code = [24, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&address_bytes);
                Ok(byte_code)
            }
            VMCommand::IfGoto(address) => {
                let address_bytes = address.to_le_bytes();
                let mut byte_code = [25, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&address_bytes);
                Ok(byte_code)
            }
            VMCommand::Function { n_locals } => {
                let n_locals_bytes = n_locals.to_le_bytes();
                let mut byte_code = [26, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&n_locals_bytes);
                Ok(byte_code)
            }
            VMCommand::Call { address, n_args } => {
                let address_bytes = address.to_le_bytes(); // [u8;4]
                let mut byte_code: [u8; 5] = [27, 0, 0, 0, 0];
                byte_code[1..4].copy_from_slice(&address_bytes[0..3]);
                byte_code[4] = n_args as u8;
                Ok(byte_code)
            }
            VMCommand::Return => Ok([28, 0, 0, 0, 0]),
            VMCommand::PushCardCount => Ok([29, 0, 0, 0, 0]),
            VMCommand::PushTypeCount => Ok([30, 0, 0, 0, 0]),
            VMCommand::PushCardType => Ok([31, 0, 0, 0, 0]),
            VMCommand::PushCardCountWithCardType => Ok([32, 0, 0, 0, 0]),
            VMCommand::PushCardTypeAttrByTypeIndex { attr_index } => {
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [33, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&attr_index_bytes);
                Ok(byte_code)
            }
            VMCommand::PushCardTypeAttrByCardIndex { attr_index } => {
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [34, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&attr_index_bytes);
                Ok(byte_code)
            }
            VMCommand::PushCardAttr { attr_index } => {
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [35, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&attr_index_bytes);
                Ok(byte_code)
            }
            VMCommand::PopCardAttr { attr_index } => {
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [36, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&attr_index_bytes);
                Ok(byte_code)
            }
            VMCommand::InstanceCardByTypeIndex => Ok([37, 0, 0, 0, 0]),
            VMCommand::InstanceCardByTypeId => Ok([38, 0, 0, 0, 0]),
            VMCommand::CallCardAction => Ok([39, 0, 0, 0, 0]),
            VMCommand::RemoveCardByIndex => Ok([40, 0, 0, 0, 0]),
            _ => {
                unimplemented!()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
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
    #[test_case(VMCommand::PushConstant(Word::Numeric(0)))]
    #[test_case(VMCommand::PushConstant(Word::Numeric(123)))]
    #[test_case(VMCommand::PushConstant(Word::Numeric(-124)))]
    #[test_case(VMCommand::PushBoardAttr{index: 123})]
    #[test_case(VMCommand::PushBoardAttr{index: 0})]
    #[test_case(VMCommand::PopBoardAttr{index: 123})]
    #[test_case(VMCommand::PopBoardAttr{index: 0})]
    #[test_case(VMCommand::PushLocal{index: 123})]
    #[test_case(VMCommand::PushLocal{index: 0})]
    #[test_case(VMCommand::PopLocal{index: 123})]
    #[test_case(VMCommand::PopLocal{index: 0})]
    #[test_case(VMCommand::PushArgument{index: 123})]
    #[test_case(VMCommand::PushArgument{index: 0})]
    #[test_case(VMCommand::PopArgument{index: 123})]
    #[test_case(VMCommand::PopArgument{index: 0})]
    #[test_case(VMCommand::Goto(0))]
    #[test_case(VMCommand::Goto(123))]
    #[test_case(VMCommand::IfGoto(0))]
    #[test_case(VMCommand::IfGoto(123))]
    #[test_case(VMCommand::Call { address: 0, n_args:0 })]
    #[test_case(VMCommand::Call { address: 2, n_args: 123 })]
    #[test_case(VMCommand::Function { n_locals: 0 })]
    #[test_case(VMCommand::Function { n_locals: 123 })]
    #[test_case(VMCommand::Return)]
    #[test_case(VMCommand::PushCardCount)]
    #[test_case(VMCommand::PushTypeCount)]
    #[test_case(VMCommand::PushCardCountWithCardType)]
    #[test_case(VMCommand::PushCardType)]
    #[test_case(VMCommand::PushCardTypeAttrByTypeIndex { attr_index: 0 })]
    #[test_case(VMCommand::PushCardTypeAttrByTypeIndex { attr_index: 123 })]
    #[test_case(VMCommand::PushCardTypeAttrByCardIndex { attr_index: 0 })]
    #[test_case(VMCommand::PushCardTypeAttrByCardIndex { attr_index: 123 })]
    #[test_case(VMCommand::PushCardAttr { attr_index: 0 })]
    #[test_case(VMCommand::PushCardAttr { attr_index: 123 })]
    #[test_case(VMCommand::PopCardAttr { attr_index: 0 })]
    #[test_case(VMCommand::PopCardAttr { attr_index: 123 })]
    #[test_case(VMCommand::InstanceCardByTypeIndex)]
    #[test_case(VMCommand::InstanceCardByTypeId)]
    #[test_case(VMCommand::CallCardAction)]
    #[test_case(VMCommand::RemoveCardByIndex)]
    fn bytecode_to_instruction_equivalence(instruction: VMCommand) {
        let bytecode = CommandByteCode::try_from(instruction).unwrap();
        let decoded_instruction = VMCommand::try_from(bytecode).unwrap();
        pretty_assertions::assert_eq!(instruction, decoded_instruction);
    }
}
