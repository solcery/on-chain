use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryFrom;
use std::convert::TryInto;

use super::word::Word;

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
    ///Pushes external value on the stack
    PushConstant(Word),
    LoadGameStateAttr {
        index: u32,
    },
    StoreGameStateAttr {
        index: u32,
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
        address: u32,
        n_args: u8,
    },
    Return,
    /// For functions that does not return a value
    ReturnVoid,

    // Interactions with cards
    /// Pushes total number of cards to the stack
    PushObjectCount,
    /// Pushes total number of card types to the stack
    PushTypeCount,
    /// Pushes [ObjectType](crate::card::ObjectType) on the `i`-th card, where `i` is the topmost value on the stack
    PushObjectType,
    /// Pushes total number of cards with [ObjectType](crate::card::ObjectType) popped from the stack
    PushObjectCountWithObjectType,
    /// Pushes `attr_index`-th attribute of the [ObjectType](crate::card::ObjectType), those index
    /// among [ObjectTypes](crate::card::ObjectType) is on the top of the stack
    LoadObjectTypeAttrByTypeIndex {
        attr_index: u32,
    },
    /// Pushes `attr_index`-th attribute of the [ObjectType](crate::card::ObjectType) of the card,
    /// those index is on the top of the stack
    LoadObjectTypeAttrByObjectIndex {
        attr_index: u32,
    },
    /// Pushes `attr_index`-th attribute of the [Object](crate::card::Object),
    /// those index is on the top of the stack
    LoadObjectAttr {
        attr_index: u32,
    },
    /// Pops `attr_index`-th attribute of the [Object](crate::card::Object),
    /// those index is on the top of the stack
    StoreObjectAttr {
        attr_index: u32,
    },

    /// Pops [ObjectType](crate::card::ObjectType) index from the stack and calls it's `action_id` action as a function
    InstanceObjectByTypeIndex,
    /// Pops [ObjectType](crate::card::ObjectType) id from the stack and calls it's `action_id` action as a function
    InstanceObjectByTypeId,
    /// ObjectType index and action index should be placed on the stack
    CallObjectAction,
    RemoveObjectByIndex,
    RemoveObjectById,
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
        let discriminant = word[0];
        match discriminant {
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
            16 => match word[1..].try_into() {
                Ok(val) => Ok(Self::PushConstant(Word::Numeric(i32::from_le_bytes(val)))),
                Err(_) => Err("PushConstant argument corrupted."),
            },
            17 => {
                let bool_data = word[1];
                match bool_data {
                    0 => Ok(Self::PushConstant(Word::Boolean(false))),
                    1 => Ok(Self::PushConstant(Word::Boolean(true))),
                    _ => Err("PushConstant argument corrupted."),
                }
            }
            18 => match word[1..].try_into() {
                Ok(val) => Ok(Self::LoadGameStateAttr {
                    index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("LoadGameStateAttr argument corrupted."),
            },
            19 => match word[1..].try_into() {
                Ok(val) => Ok(Self::StoreGameStateAttr {
                    index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("StoreGameStateAttr argument corrupted."),
            },
            20 => match word[1..].try_into() {
                Ok(val) => Ok(Self::LoadLocal {
                    index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("LoadLocal argument corrupted."),
            },
            21 => match word[1..].try_into() {
                Ok(val) => Ok(Self::StoreLocal {
                    index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("StoreLocal argument corrupted."),
            },
            22 => match word[1..].try_into() {
                Ok(val) => Ok(Self::LoadArgument {
                    index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("LoadArgument argument corrupted."),
            },
            23 => match word[1..].try_into() {
                Ok(val) => Ok(Self::StoreArgument {
                    index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("StoreArgument argument corrupted."),
            },
            24 => match word[1..].try_into() {
                Ok(val) => Ok(Self::Goto(u32::from_le_bytes(val))),
                Err(_) => Err("Goto argument corrupted."),
            },
            25 => match word[1..].try_into() {
                Ok(val) => Ok(Self::IfGoto(u32::from_le_bytes(val))),
                Err(_) => Err("IfGoto argument corrupted."),
            },
            26 => match word[1..].try_into() {
                Ok(val) => Ok(Self::Function {
                    n_locals: u32::from_le_bytes(val),
                }),
                Err(_) => Err("Function argument corrupted."),
            },
            27 => {
                // Actually, addresses are 24 bit wide, so the maximum number of instructions is
                // 2^16 (approx. 81MB total, so it is bigger than the maximum account size)
                let mut address_bytes = [0; 4];
                address_bytes[..3].clone_from_slice(&word[1..4]);

                Ok(Self::Call {
                    address: u32::from_le_bytes(address_bytes),

                    n_args: word[4],
                })
            }
            28 => Ok(Self::Return),
            29 => Ok(Self::ReturnVoid),
            30 => Ok(Self::PushObjectCount),
            31 => Ok(Self::PushTypeCount),
            32 => Ok(Self::PushObjectType),
            33 => Ok(Self::PushObjectCountWithObjectType),
            34 => match word[1..].try_into() {
                Ok(val) => Ok(Self::LoadObjectTypeAttrByTypeIndex {
                    attr_index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("LoadObjectTypeAttrByTypeIndex argument corrupted."),
            },
            35 => match word[1..].try_into() {
                Ok(val) => Ok(Self::LoadObjectTypeAttrByObjectIndex {
                    attr_index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("LoadObjectTypeAttrByObjectIndex argument corrupted."),
            },
            36 => match word[1..].try_into() {
                Ok(val) => Ok(Self::LoadObjectAttr {
                    attr_index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("LoadObjectAttr argument corrupted."),
            },
            37 => match word[1..].try_into() {
                Ok(val) => Ok(Self::StoreObjectAttr {
                    attr_index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("StoreObjectAttr argument corrupted."),
            },
            38 => Ok(Self::InstanceObjectByTypeIndex),
            39 => Ok(Self::InstanceObjectByTypeId),
            40 => Ok(Self::CallObjectAction),
            41 => Ok(Self::RemoveObjectByIndex),
            42 => Ok(Self::RemoveObjectById),
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
            VMCommand::LoadGameStateAttr { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [18, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                byte_code
            }
            VMCommand::StoreGameStateAttr { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [19, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                byte_code
            }
            VMCommand::LoadLocal { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [20, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                byte_code
            }
            VMCommand::StoreLocal { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [21, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                byte_code
            }
            VMCommand::LoadArgument { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [22, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                byte_code
            }
            VMCommand::StoreArgument { index } => {
                let index_bytes = index.to_le_bytes();
                let mut byte_code = [23, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&index_bytes);
                byte_code
            }
            VMCommand::Goto(address) => {
                let address_bytes = address.to_le_bytes();
                let mut byte_code = [24, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&address_bytes);
                byte_code
            }
            VMCommand::IfGoto(address) => {
                let address_bytes = address.to_le_bytes();
                let mut byte_code = [25, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&address_bytes);
                byte_code
            }
            VMCommand::Function { n_locals } => {
                let n_locals_bytes = n_locals.to_le_bytes();
                let mut byte_code = [26, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&n_locals_bytes);
                byte_code
            }
            VMCommand::Call { address, n_args } => {
                let address_bytes = address.to_le_bytes(); // [u8;4]
                let mut byte_code: [u8; 5] = [27, 0, 0, 0, 0];
                byte_code[1..4].copy_from_slice(&address_bytes[0..3]);
                byte_code[4] = n_args as u8;
                byte_code
            }
            VMCommand::Return => [28, 0, 0, 0, 0],
            VMCommand::ReturnVoid => [29, 0, 0, 0, 0],
            VMCommand::PushObjectCount => [30, 0, 0, 0, 0],
            VMCommand::PushTypeCount => [31, 0, 0, 0, 0],
            VMCommand::PushObjectType => [32, 0, 0, 0, 0],
            VMCommand::PushObjectCountWithObjectType => [33, 0, 0, 0, 0],
            VMCommand::LoadObjectTypeAttrByTypeIndex { attr_index } => {
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [34, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&attr_index_bytes);
                byte_code
            }
            VMCommand::LoadObjectTypeAttrByObjectIndex { attr_index } => {
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [35, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&attr_index_bytes);
                byte_code
            }
            VMCommand::LoadObjectAttr { attr_index } => {
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [36, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&attr_index_bytes);
                byte_code
            }
            VMCommand::StoreObjectAttr { attr_index } => {
                let attr_index_bytes = attr_index.to_le_bytes();
                let mut byte_code = [37, 0, 0, 0, 0];
                byte_code[1..].copy_from_slice(&attr_index_bytes);
                byte_code
            }
            VMCommand::InstanceObjectByTypeIndex => [38, 0, 0, 0, 0],
            VMCommand::InstanceObjectByTypeId => [39, 0, 0, 0, 0],
            VMCommand::CallObjectAction => [40, 0, 0, 0, 0],
            VMCommand::RemoveObjectByIndex => [41, 0, 0, 0, 0],
            VMCommand::RemoveObjectById => [42, 0, 0, 0, 0],
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
    #[test_case(VMCommand::PushConstant(Word::Numeric(0)))]
    #[test_case(VMCommand::PushConstant(Word::Numeric(123)))]
    #[test_case(VMCommand::PushConstant(Word::Numeric(-124)))]
    #[test_case(VMCommand::LoadGameStateAttr{index: 123})]
    #[test_case(VMCommand::LoadGameStateAttr{index: 0})]
    #[test_case(VMCommand::StoreGameStateAttr{index: 123})]
    #[test_case(VMCommand::StoreGameStateAttr{index: 0})]
    #[test_case(VMCommand::LoadLocal{index: 123})]
    #[test_case(VMCommand::LoadLocal{index: 0})]
    #[test_case(VMCommand::StoreLocal{index: 123})]
    #[test_case(VMCommand::StoreLocal{index: 0})]
    #[test_case(VMCommand::LoadArgument{index: 123})]
    #[test_case(VMCommand::LoadArgument{index: 0})]
    #[test_case(VMCommand::StoreArgument{index: 123})]
    #[test_case(VMCommand::StoreArgument{index: 0})]
    #[test_case(VMCommand::Goto(0))]
    #[test_case(VMCommand::Goto(123))]
    #[test_case(VMCommand::IfGoto(0))]
    #[test_case(VMCommand::IfGoto(123))]
    #[test_case(VMCommand::Call { address: 0, n_args:0 })]
    #[test_case(VMCommand::Call { address: 2, n_args: 123 })]
    #[test_case(VMCommand::Function { n_locals: 0 })]
    #[test_case(VMCommand::Function { n_locals: 123 })]
    #[test_case(VMCommand::Return)]
    #[test_case(VMCommand::ReturnVoid)]
    #[test_case(VMCommand::PushObjectCount)]
    #[test_case(VMCommand::PushTypeCount)]
    #[test_case(VMCommand::PushObjectCountWithObjectType)]
    #[test_case(VMCommand::PushObjectType)]
    #[test_case(VMCommand::LoadObjectTypeAttrByTypeIndex { attr_index: 0 })]
    #[test_case(VMCommand::LoadObjectTypeAttrByTypeIndex { attr_index: 123 })]
    #[test_case(VMCommand::LoadObjectTypeAttrByObjectIndex { attr_index: 0 })]
    #[test_case(VMCommand::LoadObjectTypeAttrByObjectIndex { attr_index: 123 })]
    #[test_case(VMCommand::LoadObjectAttr { attr_index: 0 })]
    #[test_case(VMCommand::LoadObjectAttr { attr_index: 123 })]
    #[test_case(VMCommand::StoreObjectAttr { attr_index: 0 })]
    #[test_case(VMCommand::StoreObjectAttr { attr_index: 123 })]
    #[test_case(VMCommand::InstanceObjectByTypeIndex)]
    #[test_case(VMCommand::InstanceObjectByTypeId)]
    #[test_case(VMCommand::CallObjectAction)]
    #[test_case(VMCommand::RemoveObjectByIndex)]
    #[test_case(VMCommand::RemoveObjectById)]
    fn bytecode_to_instruction_equivalence(instruction: VMCommand) {
        let bytecode = CommandByteCode::from(instruction);
        let decoded_instruction = VMCommand::try_from(bytecode).unwrap();
        pretty_assertions::assert_eq!(instruction, decoded_instruction);
    }
}
