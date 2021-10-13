use crate::word::Word;
use borsh::BorshSerialize;
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::convert::TryInto;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Deserialize, Serialize, BorshSerialize)]
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
        n_args: u32,
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
                Ok(val) => Ok(VMCommand::PushLocal {
                    index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("PushLocal argument corrupted."),
            },
            23 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::PopLocal {
                    index: u32::from_le_bytes(val),
                }),
                Err(_) => Err("PopLocal argument corrupted."),
            },
            24 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::Goto (
                    u32::from_le_bytes(val),
                )),
                Err(_) => Err("Goto argument corrupted."),
            },
            25 => match word[1..].try_into() {
                Ok(val) => Ok(VMCommand::IfGoto (
                    u32::from_le_bytes(val),
                )),
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
                let mut address_bytes = [0;4];
                address_bytes[1..].clone_from_slice(&word[1..3]);
                let n_args_bytes = [0,0,0,word[4]];


                Ok(VMCommand::Call {
                    address: u32::from_le_bytes(address_bytes),

                    n_args: u32::from_le_bytes(n_args_bytes),
                })
            },
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
        unimplemented!();
        //match instruction {
            //VMCommand::Add => {
                //self.memory.add();
                //Ok(())
            //}
            //VMCommand::Sub => {
                //self.memory.sub();
                //Ok(())
            //}
            //VMCommand::Mul => {
                //self.memory.mul();
                //Ok(())
            //}
            //VMCommand::Div => {
                //self.memory.div();
                //Ok(())
            //}
            //VMCommand::Rem => {
                //self.memory.rem();
                //Ok(())
            //}
            //VMCommand::Neg => {
                //self.memory.neg();
                //Ok(())
            //}
            //VMCommand::Inc => {
                //self.memory.inc();
                //Ok(())
            //}
            //VMCommand::Dec => {
                //self.memory.dec();
                //Ok(())
            //}
            //VMCommand::Abs => {
                //self.memory.abs();
                //Ok(())
            //}
            //VMCommand::Eq => {
                //self.memory.equal();
                //Ok(())
            //}
            //VMCommand::Gt => {
                //self.memory.gt();
                //Ok(())
            //}
            //VMCommand::Lt => {
                //self.memory.lt();
                //Ok(())
            //}
            //VMCommand::Or => {
                //self.memory.or();
                //Ok(())
            //}
            //VMCommand::And => {
                //self.memory.and();
                //Ok(())
            //}
            //VMCommand::Not => {
                //self.memory.not();
                //Ok(())
            //}
            //VMCommand::PushConstant(word) => {
                //self.memory.push_external(word);
                //Ok(())
            //}
            //VMCommand::PushBoardAttr { index } => {
                //let attr = self.board.attrs[index as usize];
                //self.memory.push_external(attr);
                //Ok(())
            //}
            //VMCommand::PopBoardAttr { index } => {
                //let value = self.memory.pop_external();
                //self.board.attrs[index as usize] = value;
                //Ok(())
            //}
            //VMCommand::PushLocal { index } => {
                //self.memory.push_local(index as usize);
                //Ok(())
            //}
            //VMCommand::PopLocal { index } => {
                //self.memory.pop_local(index as usize);
                //Ok(())
            //}
            //VMCommand::PushArgument { index } => {
                //self.memory.push_argument(index as usize);
                //Ok(())
            //}
            //VMCommand::PopArgument { index } => {
                //self.memory.pop_argument(index as usize);
                //Ok(())
            //}
            //VMCommand::Goto(instruction) => {
                //self.memory.jmp(instruction as usize);
                //Ok(())
            //}
            //VMCommand::IfGoto(instruction) => {
                //self.memory.ifjmp(instruction as usize);
                //Ok(())
            //}
            //VMCommand::Call { address, n_args } => {
                //self.memory.call(address as usize, n_args as usize);
                //Ok(())
            //}
            //VMCommand::Function { n_locals } => {
                //self.memory.function(n_locals as usize);
                //Ok(())
            //}
            //VMCommand::Return => {
                //self.memory.fn_return();
                //Ok(())
            //}
            //VMCommand::PushCardCount => {
                //let len = self.board.cards.len();
                //self.memory
                    //.push_external(Word::Numeric(TryInto::try_into(len).unwrap()));
                //Ok(())
            //}
            //VMCommand::PushTypeCount => {
                //let len = self.rom.card_type_count();
                //self.memory
                    //.push_external(Word::Numeric(TryInto::try_into(len).unwrap()));
                //Ok(())
            //}
            //VMCommand::PushCardCountWithCardType => {
                //self.push_card_count_with_type();
                //Ok(())
            //}
            //VMCommand::PushCardType => {
                //self.push_card_type();
                //Ok(())
            //}
            //VMCommand::PushCardTypeAttrByTypeIndex { attr_index } => {
                //self.push_card_type_attr_by_type_index(attr_index);
                //Ok(())
            //}
            //VMCommand::PushCardTypeAttrByCardIndex { attr_index } => {
                //self.push_card_type_attr_by_card_index(attr_index);
                //Ok(())
            //}
            //VMCommand::PushCardAttr { attr_index } => {
                //self.push_card_attr(attr_index);
                //Ok(())
            //}
            //VMCommand::PopCardAttr { attr_index } => {
                //self.pop_card_attr(attr_index);
                //Ok(())
            //}
            //VMCommand::InstanceCardByTypeIndex => {
                //self.instantiate_card_by_type_index();
                //Ok(())
            //}
            //VMCommand::InstanceCardByTypeId => {
                //self.instantiate_card_by_type_id();
                //Ok(())
            //}
            //VMCommand::CallCardAction => {
                //self.call_card_action();
                //Ok(())
            //}
            //VMCommand::RemoveCardByIndex => {
                //self.remove_card_by_index();
                //Ok(())
            //}
            //VMCommand::Halt => Err(()),
        //}
    }
}
