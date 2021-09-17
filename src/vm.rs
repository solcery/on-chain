//! # The Sorcery Virtual Machine

use crate::board::Board;
use crate::word::Word;
use tinyvec::ArrayVec;

mod memory;
use memory::Memory;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum VMCommand {
    // Arithmetic
    /// Adds two topmost values from the stack
    /// # Panics
    /// Panics if there are no enough elements on the stack or if one of the arguments is
    /// [Word::Boolean]
    Add,
    Sub,
    Div,
    Mul,
    Rem,
    Neg,
    /// Increments the topmost value on the stack
    Inc,
    /// Decrements the topmost value on the stack
    Dec,
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
        index: usize,
    },
    PopBoardAttr {
        index: usize,
    },
    PushLocal {
        index: usize,
    },
    PopLocal {
        index: usize,
    },
    PushArgument {
        index: usize,
    },
    PopArgument {
        index: usize,
    },

    // Flow control
    Goto(usize),
    IfGoto(usize),
    Halt,
    Function {
        n_locals: usize,
    },
    Call {
        address: usize,
        n_args: usize,
    },
    Return,

    // Interactions with cards
    /// Pushes total number of cards on the board to the stack
    PushDeckSize,
    /// Pushes [CardType](crate::card::CardType) on the `i`-th card, where `i` is the topmost value on the stack
    PushCardType,
    /// Pushes total number of cards with [CardType](crate::card::CardType) popped from the stack
    PushCardCountWithCardType,
    /// Pushes `attr_index`-th attribute of the [CardType](crate::card::CardType), those index
    /// among [CardTypes](crate::card::CardType) is on the top of the stack
    PushCardTypeAttrByTypeIndex {
        attr_index: usize,
    },
    /// Pushes `attr_index`-th attribute of the [CardType](crate::card::CardType) of the card,
    /// those index is on the top of the stack
    PushCardTypeAttrByCardIndex {
        attr_index: usize,
    },
    /// Pushes `attr_index`-th attribute of the [Card](crate::card::Card),
    /// those index is on the top of the stack
    PushCardAttr {
        attr_index: usize,
    },
    /// Pops `attr_index`-th attribute of the [Card](crate::card::Card),
    /// those index is on the top of the stack
    PopCardAttr {
        attr_index: usize,
    },
    /// Pushes `attr_index`-th attribute of the [Card](crate::card::Card),
    /// those [CardType](crate::card::CardType) and index is on the top of the stack
    PushCardAttrByType {
        attr_index: usize,
    },
    /// Pops `attr_index`-th attribute of the [Card](crate::card::Card),
    /// those [CardType](crate::card::CardType) and index is on the top of the stack
    PopCardAttrByType {
        attr_index: usize,
    },
}

impl Default for VMCommand {
    fn default() -> Self {
        VMCommand::Halt
    }
}

const ROM_SIZE: usize = 512;
type ROM = ArrayVec<[VMCommand; ROM_SIZE]>;

pub struct VM<'a> {
    rom: ROM,
    memory: Memory,
    board: &'a mut Board,
}

impl<'a> VM<'a> {
    pub fn new(rom: ROM, board: &'a mut Board) -> VM<'a> {
        VM {
            rom,
            memory: Memory::new(),
            board,
        }
    }
    fn run_one_instruction(&mut self) -> Result<(), ()> {
        //TODO: better handing for Halt instruction.
        //Probably, we need to propogate errors from the instructions to this function.
        let instruction = self.rom[self.memory.pc()];
        match instruction {
            VMCommand::Add => {
                self.memory.add();
                Ok(())
            }
            VMCommand::Sub => {
                self.memory.sub();
                Ok(())
            }
            VMCommand::Mul => {
                self.memory.mul();
                Ok(())
            }
            VMCommand::Div => {
                self.memory.div();
                Ok(())
            }
            VMCommand::Rem => {
                self.memory.rem();
                Ok(())
            }
            VMCommand::Neg => {
                self.memory.neg();
                Ok(())
            }
            VMCommand::Eq => {
                self.memory.eq();
                Ok(())
            }
            VMCommand::Gt => {
                self.memory.gt();
                Ok(())
            }
            VMCommand::Lt => {
                self.memory.lt();
                Ok(())
            }
            VMCommand::Or => {
                self.memory.or();
                Ok(())
            }
            VMCommand::And => {
                self.memory.and();
                Ok(())
            }
            VMCommand::Not => {
                self.memory.not();
                Ok(())
            }
            VMCommand::PushConstant(word) => {
                self.memory.push_external(word);
                Ok(())
            }
            VMCommand::PushBoardAttr { index } => {
                let attr = self.board.get_attr_by_index(index);
                self.memory.push_external(attr);
                Ok(())
            }
            VMCommand::PopBoardAttr { index } => {
                self.board.check_attr_index(index).unwrap();
                let value = self.memory.pop_external();
                self.board.set_attr_by_index(value, index);
                Ok(())
            }
            VMCommand::PushLocal { index } => {
                self.memory.push_local(index);
                Ok(())
            }
            VMCommand::PopLocal { index } => {
                self.memory.pop_local(index);
                Ok(())
            }
            VMCommand::PushArgument { index } => {
                self.memory.push_argument(index);
                Ok(())
            }
            VMCommand::PopArgument { index } => {
                self.memory.pop_argument(index);
                Ok(())
            }
            VMCommand::Goto(instruction) => {
                self.memory.jmp(instruction);
                Ok(())
            }
            VMCommand::IfGoto(instruction) => {
                self.memory.ifjmp(instruction);
                Ok(())
            }
            VMCommand::Call { address, n_args } => {
                self.memory.call(address, n_args);
                Ok(())
            }
            VMCommand::Function { n_locals } => {
                self.memory.function(n_locals);
                Ok(())
            }
            VMCommand::Return => {
                self.memory.fn_return();
                Ok(())
            }
            VMCommand::Halt => Err(()),
            _ => {
                unimplemented!();
            }
        }
    }

    pub fn execute(&mut self, instruction_limit: usize) {
        for _ in 0..instruction_limit {
            if self.run_one_instruction().is_err() {
                break;
            }
        }
    }
}
