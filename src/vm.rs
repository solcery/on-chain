//! # The Sorcery Virtual Machine

use crate::board::{Board, Error as BoardError};
use crate::instruction_rom::InstructionRom;
use crate::rom::CardTypesRom;
use crate::vmcommand::VMCommand;
use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryFrom;
use std::convert::TryInto;

mod memory;
use memory::Error as MemoryError;
use memory::Memory;

mod log;
use log::{Event, Log};

mod enums;
use enums::ExecutionResult;
pub use enums::SingleExecutionResult;

mod error;
pub use error::{Error, ErrorSource};

#[derive(Debug, Clone, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Sealed<T> {
    data: T,
}

impl<T> Sealed<T> {
    fn release_data(self) -> T {
        self.data
    }
}

#[derive(Debug)]
pub struct VM<'a, Brd: Board> {
    instructions: InstructionRom<'a>,
    card_types: CardTypesRom<'a>,
    memory: Memory,
    board: &'a mut Brd,
    log: Log,
}

impl<'a, Brd: Board> VM<'a, Brd> {
    pub fn init_vm(
        instructions: InstructionRom<'a>,
        card_types: CardTypesRom<'a>,
        board: &'a mut Brd,
        args: &'a [Word],
        card_index: u32,
        action_index: u32,
    ) -> Self {
        let memory = Memory::init_memory(args, card_index, action_index);
        Self {
            instructions,
            card_types,
            memory,
            board,
            log: vec![],
        }
    }

    pub fn execute(&mut self, instruction_limit: usize) -> Result<SingleExecutionResult, Error> {
        for _ in 0..instruction_limit {
            match self.run_one_instruction() {
                Ok(()) => {}
                Err(err) => match err {
                    ErrorSource::Halt => {
                        return Ok(SingleExecutionResult::Finished);
                    }
                    err => {
                        // TODO: Should be changed with Error trait
                        let error = Error {
                            instruction: self.memory.pc() as u32,
                            source: err,
                        };
                        return Err(error);
                    }
                },
            }
        }
        Ok(SingleExecutionResult::Unfinished)
    }

    pub fn resume_execution(
        instructions: InstructionRom<'a>,
        card_types: CardTypesRom<'a>,
        board: &'a mut Brd,
        sealed_memory: Sealed<Memory>,
    ) -> Self {
        let memory = Sealed::<Memory>::release_data(sealed_memory);
        Self {
            instructions,
            card_types,
            memory,
            board,
            log: vec![],
        }
    }

    #[must_use]
    pub fn stop_execution(self) -> ExecutionResult {
        if self.is_halted() {
            ExecutionResult::Finished(self.log)
        } else {
            ExecutionResult::Unfinished(self.log, Sealed::<Memory> { data: self.memory })
        }
    }

    fn run_one_instruction(&mut self) -> Result<(), ErrorSource> {
        //TODO: better handing for Halt instruction.
        //Probably, we need to propogate MemoryErrors from the instructions to this function.
        let instruction = self.instructions.fetch_instruction(self.memory.pc());
        match instruction {
            VMCommand::Add => self.memory.add().map_err(ErrorSource::from),
            VMCommand::Sub => self.memory.sub().map_err(ErrorSource::from),
            VMCommand::Mul => self.memory.mul().map_err(ErrorSource::from),
            VMCommand::Div => self.memory.div().map_err(ErrorSource::from),
            VMCommand::Rem => self.memory.rem().map_err(ErrorSource::from),
            VMCommand::Neg => self.memory.neg().map_err(ErrorSource::from),
            VMCommand::Inc => self.memory.inc().map_err(ErrorSource::from),
            VMCommand::Dec => self.memory.dec().map_err(ErrorSource::from),
            VMCommand::Abs => self.memory.abs().map_err(ErrorSource::from),
            VMCommand::Eq => self.memory.equal().map_err(ErrorSource::from),
            VMCommand::Gt => self.memory.gt().map_err(ErrorSource::from),
            VMCommand::Lt => self.memory.lt().map_err(ErrorSource::from),
            VMCommand::Or => self.memory.or().map_err(ErrorSource::from),
            VMCommand::And => self.memory.and().map_err(ErrorSource::from),
            VMCommand::Not => self.memory.not().map_err(ErrorSource::from),
            VMCommand::PushConstant(word) => {
                self.memory.push_external(word).map_err(ErrorSource::from)
            }
            VMCommand::LoadRegionAttr {
                region_index,
                attr_index,
            } => {
                // TODO: Error handing
                match self.board.memory_region(region_index as usize) {
                    Ok(region) => {
                        let attr = region.attrs[attr_index as usize];
                        self.memory.push_external(attr).map_err(ErrorSource::from)
                    }
                    Err(board_error) => todo!(),
                }
            }
            VMCommand::StoreRegionAttr {
                region_index,
                attr_index,
            } => {
                // TODO: Error handing
                let value = self.memory.pop_external().map_err(ErrorSource::from)?;
                match self.board.memory_region(region_index as usize) {
                    Ok(region) => {
                        self.log.push(Event::RegionChange {
                            region_index,
                            attr_index: attr_index,
                            previous_value: region.attrs[attr_index as usize],
                            new_value: value,
                        });
                        region.attrs[attr_index as usize] = value;
                        Ok(())
                    }
                    Err(board_error) => todo!(),
                }
            }
            VMCommand::LoadLocal { index } => self
                .memory
                .push_local(index as usize)
                .map_err(ErrorSource::from),
            VMCommand::StoreLocal { index } => self
                .memory
                .pop_local(index as usize)
                .map_err(ErrorSource::from),
            VMCommand::LoadArgument { index } => self
                .memory
                .push_argument(index as usize)
                .map_err(ErrorSource::from),
            VMCommand::StoreArgument { index } => self
                .memory
                .pop_argument(index as usize)
                .map_err(ErrorSource::from),
            VMCommand::Goto(instruction) => self
                .memory
                .jmp(instruction as usize)
                .map_err(ErrorSource::from),
            VMCommand::IfGoto(instruction) => self
                .memory
                .ifjmp(instruction as usize)
                .map_err(ErrorSource::from),
            VMCommand::Call { address, n_args } => self
                .memory
                .call(address as usize, n_args as usize)
                .map_err(ErrorSource::from),
            VMCommand::Function { n_locals } => self
                .memory
                .function(n_locals as usize)
                .map_err(ErrorSource::from),
            VMCommand::Return => self.memory.fn_return().map_err(ErrorSource::from),
            VMCommand::ReturnVoid => self.memory.return_void().map_err(ErrorSource::from),
            VMCommand::PushCardCount { region_index } => {
                // TODO: Error handing
                match self.board.memory_region(region_index as usize) {
                    Ok(region) => {
                        let len = region.cards.len();
                        self.memory
                            .push_external(Word::Numeric(len as i32))
                            .map_err(ErrorSource::from)
                    }
                    Err(board_error) => todo!(),
                }
            }
            VMCommand::PushTypeCount => {
                let len = self.card_types.card_type_count();
                self.memory
                    .push_external(Word::Numeric(len as i32))
                    .map_err(ErrorSource::from)
            }
            VMCommand::PushCardCountWithCardType { region_index } => {
                self.push_card_count_with_type(region_index)
            }
            VMCommand::PushCardTypeByCardIndex { region_index } => {
                self.push_card_type_by_index(region_index)
            }
            VMCommand::PushCardTypeByCardId => unimplemented!(),
            VMCommand::LoadCardTypeAttrByTypeIndex { attr_index } => self
                .push_card_type_attr_by_type_index(attr_index)
                .map_err(ErrorSource::from),
            VMCommand::LoadCardTypeAttrByCardIndex {
                region_index,
                attr_index,
            } => self.push_card_type_attr_by_card_index(region_index, attr_index),
            VMCommand::LoadCardTypeAttrByCardId { attr_index } => unimplemented!(),
            VMCommand::LoadCardAttrByCardIndex {
                region_index,
                attr_index,
            } => self.push_card_attr_by_index(region_index, attr_index),
            VMCommand::LoadCardAttrByCardId { attr_index } => unimplemented!(),
            VMCommand::StoreCardAttrByCardIndex {
                region_index,
                attr_index,
            } => self.pop_card_attr(region_index, attr_index),
            VMCommand::StoreCardAttrByCardId { attr_index } => unimplemented!(),
            VMCommand::InstanceCardByTypeIndex { region_index } => {
                self.instantiate_card_by_type_index(region_index)
            }
            VMCommand::InstanceCardByTypeId { region_index } => {
                self.instantiate_card_by_type_id(region_index)
            }
            VMCommand::CallCardAction => self.call_card_action().map_err(ErrorSource::from),
            VMCommand::RemoveCardByIndex { region_index } => {
                self.remove_card_by_index(region_index)
            }
            VMCommand::RemoveCardById => self.remove_card_by_id(),
            VMCommand::Halt => Err(ErrorSource::Halt),
            VMCommand::ReadPlayerInput => unimplemented!(),
            VMCommand::ReadRandomInput => unimplemented!(),
        }
    }

    fn push_card_type_by_index(&mut self, region_index: u32) -> Result<(), ErrorSource> {
        let index = self.memory.pop_external_no_pc_inc()?;
        match index {
            Word::Numeric(i) => {
                match self.board.memory_region(region_index as usize) {
                    Ok(region) => {
                        let card_type = region.cards[i as usize].card_type();
                        //TODO: Error handing for Index out of Bounds
                        let word = Word::Numeric(card_type as i32);

                        self.memory.push_external(word).map_err(ErrorSource::from)
                    }
                    Err(board_error) => todo!(),
                }
            }
            Word::Boolean(_) => Err(ErrorSource::from(MemoryError::TypeMismatch)),
        }
    }

    fn push_card_count_with_type(&mut self, region_index: u32) -> Result<(), ErrorSource> {
        let card_type = self
            .memory
            .pop_external_no_pc_inc()
            .map_err(ErrorSource::from)?;
        match card_type {
            Word::Numeric(id) => {
                match self.board.memory_region(region_index as usize) {
                    Ok(region) => {
                        // Word::Numeric contains i32, but card_type is u32, so convert is needed
                        let count = region
                            .cards
                            .iter()
                            .filter(|card| card.card_type() == id as u32)
                            .count();

                        let word = Word::Numeric(count as i32);
                        self.memory.push_external(word).map_err(ErrorSource::from)
                    }
                    Err(board_error) => todo!(),
                }
            }
            Word::Boolean(_) => Err(ErrorSource::from(MemoryError::TypeMismatch)),
        }
    }

    fn push_card_type_attr_by_type_index(&mut self, attr_index: u32) -> Result<(), MemoryError> {
        let type_index = self.memory.pop_external_no_pc_inc()?;
        match type_index {
            Word::Numeric(id) => {
                let card_type = self.card_types.card_type_by_type_index(id as usize);
                let attr_value = card_type.attr_by_index(attr_index as usize);

                let word = attr_value;
                self.memory.push_external(word)
            }
            Word::Boolean(_) => Err(MemoryError::TypeMismatch),
        }
    }

    fn push_card_type_attr_by_card_index(
        &mut self,
        region_index: u16,
        attr_index: u16,
    ) -> Result<(), ErrorSource> {
        let card_index = self
            .memory
            .pop_external_no_pc_inc()
            .map_err(ErrorSource::from)?;
        match card_index {
            Word::Numeric(id) => {
                match self.board.memory_region(region_index as usize) {
                    Ok(region) => {
                        let card = &region.cards[id as usize];
                        //TODO: Error handing for Index out of Bounds
                        let card_type_id = card.card_type();
                        let card_type = self
                            .card_types
                            .card_type_by_type_id(card_type_id)
                            .ok_or(ErrorSource::NoSuchType)?;
                        let attr_value = card_type.attr_by_index(attr_index as usize);

                        let word = attr_value;
                        self.memory.push_external(word).map_err(ErrorSource::from)
                    }
                    Err(board_error) => todo!(),
                }
            }
            Word::Boolean(_) => Err(ErrorSource::from(MemoryError::TypeMismatch)),
        }
    }

    fn push_card_attr_by_index(
        &mut self,
        region_index: u16,
        attr_index: u16,
    ) -> Result<(), ErrorSource> {
        let card_index = self
            .memory
            .pop_external_no_pc_inc()
            .map_err(ErrorSource::from)?;
        match card_index {
            Word::Numeric(id) => match self.board.memory_region(region_index as usize) {
                Ok(region) => {
                    let card = &region.cards[id as usize];
                    let attr_value = card.attrs[attr_index as usize];

                    let word = attr_value;
                    self.memory.push_external(word).map_err(ErrorSource::from)
                }
                Err(board_error) => todo!(),
            },
            Word::Boolean(_) => Err(ErrorSource::from(MemoryError::TypeMismatch)),
        }
    }

    fn pop_card_attr(&mut self, region_index: u16, attr_index: u16) -> Result<(), ErrorSource> {
        let card_index = self
            .memory
            .pop_external_no_pc_inc()
            .map_err(ErrorSource::from)?;
        match card_index {
            Word::Numeric(id) => match self.board.memory_region(region_index as usize) {
                Ok(region) => {
                    let card = &mut region.cards[id as usize];
                    let attr_value = self.memory.pop_external().map_err(ErrorSource::from)?;

                    self.log.push(Event::CardChange {
                        region_index,
                        card_index: id as u32,
                        attr_index,
                        previous_value: card.attrs[attr_index as usize],
                        new_value: attr_value,
                    });

                    card.attrs[attr_index as usize] = attr_value;
                    Ok(())
                }
                Err(board_error) => todo!(),
            },
            Word::Boolean(_) => Err(ErrorSource::from(MemoryError::TypeMismatch)),
        }
    }

    fn instantiate_card_by_type_index(&mut self, region_index: u16) -> Result<(), ErrorSource> {
        let index = self.memory.pop_external().map_err(ErrorSource::from)?;
        match index {
            Word::Numeric(index) => {
                let new_card_id = self.board.generate_card_id();

                match self.board.memory_region(region_index as usize) {
                    //TODO: Error handing: get rid of unwraps
                    Ok(region) => {
                        let id = index.try_into().unwrap();
                        let card = self
                            .card_types
                            .instance_card_by_type_index(id, new_card_id)
                            .unwrap();
                        region.cards.push(card);

                        self.log.push(Event::AddCardByIndex {
                            card_index: (region.cards.len() - 1) as u32,
                            cardtype_index: id as u32,
                        });
                        Ok(())
                    }
                    Err(board_error) => todo!(),
                }
            }
            Word::Boolean(_) => Err(ErrorSource::from(MemoryError::TypeMismatch)),
        }
    }

    fn instantiate_card_by_type_id(&mut self, region_index: u16) -> Result<(), ErrorSource> {
        let index = self.memory.pop_external().map_err(ErrorSource::from)?;
        match index {
            Word::Numeric(index) => {
                let new_card_id = self.board.generate_card_id();

                match self.board.memory_region(region_index as usize) {
                    Ok(region) => {
                        //TODO: Error handing: get rid of unwraps
                        let id = index.try_into().unwrap();
                        let card = self
                            .card_types
                            .instance_card_by_type_id(id, new_card_id)
                            .unwrap();
                        region.cards.push(card);

                        self.log.push(Event::AddCardById {
                            card_index: (region.cards.len() - 1) as u32,
                            cardtype_id: id as u32,
                        });
                        Ok(())
                    }
                    Err(board_error) => todo!(),
                }
            }
            Word::Boolean(_) => Err(ErrorSource::from(MemoryError::TypeMismatch)),
        }
    }

    fn call_card_action(&mut self) -> Result<(), MemoryError> {
        let action_index_word = self.memory.pop_external_no_pc_inc()?;
        let action_index =
            usize::try_from(action_index_word).map_err(|_| MemoryError::TypeMismatch)?;

        let type_index_word = self.memory.pop_external_no_pc_inc()?;
        let type_index = usize::try_from(type_index_word).map_err(|_| MemoryError::TypeMismatch)?;

        let card_type = self.card_types.card_type_by_type_index(type_index);
        let entry_point = card_type.action_entry_point(action_index);
        self.memory
            .call(entry_point.address(), entry_point.n_args())?;

        self.log.push(Event::CardActionStarted {
            cardtype_index: type_index as u32,
            action_index: action_index as u32,
            args: self.memory.args(),
        });
        Ok(())
    }

    fn remove_card_by_index(&mut self, region_index: u16) -> Result<(), ErrorSource> {
        match self.board.memory_region(region_index as usize) {
            Ok(region) => {
                let card_index_word = self.memory.pop_external().map_err(ErrorSource::from)?;
                let card_index = usize::try_from(card_index_word)
                    .map_err(|_| ErrorSource::from(MemoryError::TypeMismatch))?;

                let card = region.cards.remove(card_index);

                self.log.push(Event::RemoveCard {
                    card_id: card.id() as u32,
                });
                Ok(())
            }
            Err(board_error) => todo!(),
        }
    }

    fn remove_card_by_id(&mut self) -> Result<(), ErrorSource> {
        //TODO: implement a way to know, if removal was successful or not
        let card_id = self.memory.pop_external().map_err(ErrorSource::from)?;
        let card_id =
            u32::try_from(card_id).map_err(|_| ErrorSource::from(MemoryError::TypeMismatch))?;

        let log = &mut self.log;

        for region in self.board.iter_mut() {
            region.cards.retain(|card| {
                if card.id() == card_id {
                    log.push(Event::RemoveCard {
                        card_id: card_id as u32,
                    });
                    true
                } else {
                    false
                }
            });
        }
        Ok(())
    }

    #[cfg(test)]
    fn release_memory(self) -> Memory {
        self.memory
    }

    #[must_use]
    pub fn is_halted(&self) -> bool {
        let instruction = self.instructions.fetch_instruction(self.memory.pc());
        instruction == VMCommand::Halt
    }
}

#[cfg(test)]
mod tests;
