//! # The Sorcery Virtual Machine

use super::on_chain_types::{
    game_state::GameState, instruction_rom::InstructionRom, object_type_rom::ObjectTypesRom,
    vmcommand::VMCommand, word::Word,
};
use borsh::{BorshDeserialize, BorshSerialize};
use std::convert::TryFrom;
use std::convert::TryInto;

mod memory;
use memory::Error as InternalError;
use memory::Memory;

mod log;
use log::{Event, Log};

mod enums;
use enums::ExecutionResult;
pub use enums::SingleExecutionResult;

mod error;
pub use error::Error;

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
pub struct VM<'a> {
    instructions: InstructionRom<'a>,
    object_types: ObjectTypesRom<'a>,
    memory: Memory,
    game_state: &'a mut GameState,
    log: Log,
}

impl<'a> VM<'a> {
    pub fn init_vm(
        instructions: InstructionRom<'a>,
        object_types: ObjectTypesRom<'a>,
        game_state: &'a mut GameState,
        args: &'a [Word],
        object_index: u32,
        action_index: u32,
    ) -> Self {
        let memory = Memory::init_memory(args, object_index, action_index);
        Self {
            instructions,
            object_types,
            memory,
            game_state,
            log: vec![],
        }
    }

    pub fn execute(&mut self, instruction_limit: usize) -> Result<SingleExecutionResult, Error> {
        for _ in 0..instruction_limit {
            match self.run_one_instruction() {
                Ok(()) => {}
                Err(err) => match err {
                    InternalError::Halt => {
                        return Ok(SingleExecutionResult::Finished);
                    }
                    err => {
                        // Should be changed with Error trait
                        let error = Error {
                            instruction: self.memory.pc() as u32,
                            error: err,
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
        object_types: ObjectTypesRom<'a>,
        game_state: &'a mut GameState,
        sealed_memory: Sealed<Memory>,
    ) -> Self {
        let memory = Sealed::<Memory>::release_data(sealed_memory);
        Self {
            instructions,
            object_types,
            memory,
            game_state,
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

    fn run_one_instruction(&mut self) -> Result<(), InternalError> {
        //TODO: better handing for Halt instruction.
        //Probably, we need to propogate InternalErrors from the instructions to this function.
        let instruction = self.instructions.fetch_instruction(self.memory.pc());
        match instruction {
            VMCommand::Add => self.memory.add(),
            VMCommand::Sub => self.memory.sub(),
            VMCommand::Mul => self.memory.mul(),
            VMCommand::Div => self.memory.div(),
            VMCommand::Rem => self.memory.rem(),
            VMCommand::Neg => self.memory.neg(),
            VMCommand::Inc => self.memory.inc(),
            VMCommand::Dec => self.memory.dec(),
            VMCommand::Abs => self.memory.abs(),
            VMCommand::Eq => self.memory.equal(),
            VMCommand::Gt => self.memory.gt(),
            VMCommand::Lt => self.memory.lt(),
            VMCommand::Or => self.memory.or(),
            VMCommand::And => self.memory.and(),
            VMCommand::Not => self.memory.not(),
            VMCommand::PushConstant(word) => self.memory.push_external(word),
            VMCommand::LoadGameStateAttr { index } => {
                let attr = self.game_state.attrs[index as usize];
                self.memory.push_external(attr)
            }
            VMCommand::StoreGameStateAttr { index } => {
                let value = self.memory.pop_external()?;
                self.log.push(Event::GameStateChange {
                    attr_index: index,
                    previous_value: self.game_state.attrs[index as usize],
                    new_value: value,
                });
                self.game_state.attrs[index as usize] = value;
                Ok(())
            }
            VMCommand::LoadLocal { index } => self.memory.push_local(index as usize),
            VMCommand::StoreLocal { index } => self.memory.pop_local(index as usize),
            VMCommand::LoadArgument { index } => self.memory.push_argument(index as usize),
            VMCommand::StoreArgument { index } => self.memory.pop_argument(index as usize),
            VMCommand::Goto(instruction) => self.memory.jmp(instruction as usize),
            VMCommand::IfGoto(instruction) => self.memory.ifjmp(instruction as usize),
            VMCommand::Call { address, n_args } => {
                self.memory.call(address as usize, n_args as usize)
            }
            VMCommand::Function { n_locals } => self.memory.function(n_locals as usize),
            VMCommand::Return => self.memory.fn_return(),
            VMCommand::ReturnVoid => self.memory.return_void(),
            VMCommand::PushObjectCount => {
                let len = self.game_state.objects.len();
                self.memory.push_external(Word::Numeric(len as i32))
            }
            VMCommand::PushTypeCount => {
                let len = self.object_types.object_type_count();
                self.memory.push_external(Word::Numeric(len as i32))
            }
            VMCommand::PushObjectCountWithObjectType => self.push_object_count_with_type(),
            VMCommand::PushObjectType => self.push_object_type(),
            VMCommand::LoadObjectTypeAttrByTypeIndex { attr_index } => {
                self.push_object_type_attr_by_type_index(attr_index)
            }
            VMCommand::LoadObjectTypeAttrByObjectIndex { attr_index } => {
                self.push_object_type_attr_by_object_index(attr_index)
            }
            VMCommand::LoadObjectAttr { attr_index } => self.push_object_attr(attr_index),
            VMCommand::StoreObjectAttr { attr_index } => self.pop_object_attr(attr_index),
            VMCommand::InstanceObjectByTypeIndex => self.instantiate_object_by_type_index(),
            VMCommand::InstanceObjectByTypeId => self.instantiate_object_by_type_id(),
            VMCommand::CallObjectAction => self.call_object_action(),
            VMCommand::RemoveObjectByIndex => self.remove_object_by_index(),
            VMCommand::RemoveObjectById => self.remove_object_by_id(),
            VMCommand::Halt => Err(InternalError::Halt),
        }
    }

    fn push_object_type(&mut self) -> Result<(), InternalError> {
        let index = self.memory.pop_external_no_pc_inc()?;
        match index {
            Word::Numeric(i) => {
                let object_type = self.game_state.objects[i as usize].object_type();
                let word = Word::Numeric(object_type as i32);
                self.memory.push_external(word)
            }
            Word::Boolean(_) => Err(InternalError::TypeMismatch),
        }
    }

    fn push_object_count_with_type(&mut self) -> Result<(), InternalError> {
        let object_type = self.memory.pop_external_no_pc_inc()?;
        match object_type {
            Word::Numeric(id) => {
                // Word::Numeric contains i32, but object_type is u32, so convert is needed
                let count = self
                    .game_state
                    .objects
                    .iter()
                    .filter(|object| object.object_type() == id as u32)
                    .count();

                let word = Word::Numeric(count as i32);
                self.memory.push_external(word)
            }
            Word::Boolean(_) => Err(InternalError::TypeMismatch),
        }
    }

    fn push_object_type_attr_by_type_index(
        &mut self,
        attr_index: u32,
    ) -> Result<(), InternalError> {
        let type_index = self.memory.pop_external_no_pc_inc()?;
        match type_index {
            Word::Numeric(id) => {
                let object_type = self.object_types.object_type_by_type_index(id as usize);
                let attr_value = object_type.attr_by_index(attr_index as usize);

                let word = attr_value;
                self.memory.push_external(word)
            }
            Word::Boolean(_) => Err(InternalError::TypeMismatch),
        }
    }

    fn push_object_type_attr_by_object_index(
        &mut self,
        attr_index: u32,
    ) -> Result<(), InternalError> {
        let object_index = self.memory.pop_external_no_pc_inc()?;
        match object_index {
            Word::Numeric(id) => {
                let object = &self.game_state.objects[id as usize];
                let object_type_id = object.object_type();
                let object_type = self
                    .object_types
                    .object_type_by_type_id(object_type_id)
                    .ok_or(InternalError::NoSuchType)?;
                let attr_value = object_type.attr_by_index(attr_index as usize);

                let word = attr_value;
                self.memory.push_external(word)
            }
            Word::Boolean(_) => Err(InternalError::TypeMismatch),
        }
    }

    fn push_object_attr(&mut self, attr_index: u32) -> Result<(), InternalError> {
        let object_index = self.memory.pop_external_no_pc_inc()?;
        match object_index {
            Word::Numeric(id) => {
                let object = &self.game_state.objects[id as usize];
                let attr_value = object.attrs[attr_index as usize];

                let word = attr_value;
                self.memory.push_external(word)
            }
            Word::Boolean(_) => {
                panic!("Type mismath: bool can not be interpreted as index.");
            }
        }
    }

    fn pop_object_attr(&mut self, attr_index: u32) -> Result<(), InternalError> {
        let object_index = self.memory.pop_external_no_pc_inc()?;
        match object_index {
            Word::Numeric(id) => {
                let object = &mut self.game_state.objects[id as usize];
                let attr_value = self.memory.pop_external()?;

                self.log.push(Event::ObjectChange {
                    object_index: id as u32,
                    attr_index,
                    previous_value: object.attrs[attr_index as usize],
                    new_value: attr_value,
                });

                object.attrs[attr_index as usize] = attr_value;
                Ok(())
            }
            Word::Boolean(_) => Err(InternalError::TypeMismatch),
        }
    }

    fn instantiate_object_by_type_index(&mut self) -> Result<(), InternalError> {
        let index = self.memory.pop_external()?;
        match index {
            Word::Numeric(index) => {
                let id = index.try_into().unwrap();
                let object = self
                    .object_types
                    .instance_object_by_type_index(id, self.game_state.generate_object_id())
                    .unwrap();
                self.game_state.objects.push(object);

                self.log.push(Event::AddObjectByIndex {
                    object_index: (self.game_state.objects.len() - 1) as u32,
                    object_type_index: id,
                });
                Ok(())
            }
            Word::Boolean(_) => Err(InternalError::TypeMismatch),
        }
    }

    fn instantiate_object_by_type_id(&mut self) -> Result<(), InternalError> {
        let index = self.memory.pop_external()?;
        match index {
            Word::Numeric(index) => {
                let id = index.try_into().unwrap();
                let object = self
                    .object_types
                    .instance_object_by_type_id(id, self.game_state.generate_object_id())
                    .unwrap();
                self.game_state.objects.push(object);

                self.log.push(Event::AddObjectById {
                    object_index: (self.game_state.objects.len() - 1) as u32,
                    object_type_id: id,
                });
                Ok(())
            }
            Word::Boolean(_) => Err(InternalError::TypeMismatch),
        }
    }

    fn call_object_action(&mut self) -> Result<(), InternalError> {
        let action_index_word = self.memory.pop_external_no_pc_inc()?;
        let action_index =
            usize::try_from(action_index_word).map_err(|_| InternalError::TypeMismatch)?;

        let type_index_word = self.memory.pop_external_no_pc_inc()?;
        let type_index =
            usize::try_from(type_index_word).map_err(|_| InternalError::TypeMismatch)?;

        let object_type = self.object_types.object_type_by_type_index(type_index);
        let entry_point = object_type.action_entry_point(action_index);
        self.memory
            .call(entry_point.address(), entry_point.n_args())?;

        self.log.push(Event::ObjectActionStarted {
            object_type_index: type_index as u32,
            action_index: action_index as u32,
            args: self.memory.args(),
        });
        Ok(())
    }

    fn remove_object_by_index(&mut self) -> Result<(), InternalError> {
        let object_index_word = self.memory.pop_external()?;
        let object_index =
            usize::try_from(object_index_word).map_err(|_| InternalError::TypeMismatch)?;

        let object = self.game_state.objects.remove(object_index);

        self.log.push(Event::RemoveObject {
            object_id: object.id(),
        });
        Ok(())
    }

    fn remove_object_by_id(&mut self) -> Result<(), InternalError> {
        let object_id = self.memory.pop_external()?;
        let object_id = u32::try_from(object_id).map_err(|_| InternalError::TypeMismatch)?;

        let game_state = &mut self.game_state;
        let log = &mut self.log;

        game_state.objects.retain(|object| {
            if object.id() == object_id {
                log.push(Event::RemoveObject { object_id });
                true
            } else {
                false
            }
        });

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
mod tests {
    use super::*;
    use crate::on_chain_types::object_type::{EntryPoint, ObjectType};
    use crate::word_vec;
    use pretty_assertions::assert_eq;

    fn type1() -> ObjectType {
        let type1_attrs = word_vec![10, 5, true, false,];
        let type1_init_attrs = word_vec![5, 5, false, false,];
        unsafe {
            ObjectType::from_raw_parts(
                1,
                type1_attrs,
                type1_init_attrs,
                vec![EntryPoint::from_raw_parts(2, 0)],
            )
        }
    }

    fn type2() -> ObjectType {
        let type2_attrs = word_vec![20, 5, true, true,];
        let type2_init_attrs = word_vec![6, 4, false, false,];
        unsafe {
            ObjectType::from_raw_parts(
                2,
                type2_attrs,
                type2_init_attrs,
                vec![EntryPoint::from_raw_parts(4, 0)],
            )
        }
    }

    fn testing_game_state() -> GameState {
        let type1 = type1();
        let type2 = type2();

        let game_state_attrs = word_vec![3, 4, 5, false, false, true,];

        let mut object1 = type1.instantiate_object(1);
        let mut object2 = type2.instantiate_object(2);

        object1.attrs[0] = Word::Numeric(4);
        object2.attrs[3] = Word::Boolean(true);

        let object3 = type1.instantiate_object(3);
        let object4 = type2.instantiate_object(4);

        unsafe {
            GameState::from_raw_parts(
                vec![object1, object2, object3, object4],
                game_state_attrs,
                5,
            )
        }
    }

    fn initial_game_state() -> GameState {
        let object1 = type1().instantiate_object(1);
        let game_state_attrs = word_vec![0, 0, 0, false, false, false,];
        unsafe { GameState::from_raw_parts(vec![object1], game_state_attrs, 1) }
    }

    #[test]
    fn init_empty_memory_vm() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::PushObjectType,
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let object_types = vec![type1(), type2()];
        let object_types = unsafe { ObjectTypesRom::from_raw_parts(&object_types) };

        let mut game_state = testing_game_state();

        let args = vec![];
        let vm = VM::init_vm(instructions, object_types, &mut game_state, &args, 0, 0);
        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 0, 0, 0) };

        assert_eq!(memory, needed_memory);
    }

    #[test]
    fn push_type() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::PushObjectType,
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let object_types = vec![type1(), type2()];
        let object_types = unsafe { ObjectTypesRom::from_raw_parts(&object_types) };

        let mut game_state = testing_game_state();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, object_types, &mut game_state, &args, 0, 0);

        vm.execute(10).unwrap();
        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, 1], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let game_state_needed = testing_game_state();

        assert_eq!(game_state, game_state_needed);
    }

    #[test]
    fn push_object_count() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::PushObjectCountWithObjectType,
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let object_types = vec![type1(), type2()];
        let object_types = unsafe { ObjectTypesRom::from_raw_parts(&object_types) };

        let mut game_state = testing_game_state();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, object_types, &mut game_state, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, 2], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let game_state_needed = testing_game_state();

        assert_eq!(game_state, game_state_needed);
    }

    #[test]
    fn push_type_attr_by_type_index() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(1)),
            VMCommand::LoadObjectTypeAttrByTypeIndex { attr_index: 3 },
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let object_types = vec![type1(), type2()];
        let object_types = unsafe { ObjectTypesRom::from_raw_parts(&object_types) };

        let mut game_state = testing_game_state();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, object_types, &mut game_state, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let game_state_needed = testing_game_state();

        assert_eq!(game_state, game_state_needed);
    }

    #[test]
    fn push_type_attr_by_object_index() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(1)),
            VMCommand::LoadObjectTypeAttrByObjectIndex { attr_index: 3 },
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let object_types = vec![type1(), type2()];
        let object_types = unsafe { ObjectTypesRom::from_raw_parts(&object_types) };

        let mut game_state = testing_game_state();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, object_types, &mut game_state, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let game_state_needed = testing_game_state();

        assert_eq!(game_state, game_state_needed);
    }

    #[test]
    fn push_attr() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(1)),
            VMCommand::LoadObjectAttr { attr_index: 3 },
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let object_types = vec![type1(), type2()];
        let object_types = unsafe { ObjectTypesRom::from_raw_parts(&object_types) };

        let mut game_state = testing_game_state();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, object_types, &mut game_state, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let game_state_needed = testing_game_state();

        assert_eq!(game_state, game_state_needed);
    }

    #[test]
    fn pop_attr() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(42)),
            VMCommand::PushConstant(Word::Numeric(1)),
            VMCommand::StoreObjectAttr { attr_index: 3 },
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let object_types = vec![type1(), type2()];
        let object_types = unsafe { ObjectTypesRom::from_raw_parts(&object_types) };

        let mut game_state = testing_game_state();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, object_types, &mut game_state, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 3, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut game_state_needed = testing_game_state();
        game_state_needed.objects[1].attrs[3] = Word::Numeric(42);

        assert_eq!(game_state, game_state_needed);
    }

    #[test]
    fn add_one_object_by_index() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(1)),
            VMCommand::InstanceObjectByTypeIndex,
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let object_types = vec![type1(), type2()];
        let object_types = unsafe { ObjectTypesRom::from_raw_parts(&object_types) };

        let mut game_state = initial_game_state();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, object_types, &mut game_state, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut game_state_needed = initial_game_state();
        let _ = game_state_needed.generate_object_id();
        let added_object = type2().instantiate_object(1);
        game_state_needed.objects.push(added_object);

        assert_eq!(game_state, game_state_needed);
    }

    #[test]
    fn add_one_object_by_id() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::InstanceObjectByTypeId,
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let object_types = vec![type1(), type2()];
        let object_types = unsafe { ObjectTypesRom::from_raw_parts(&object_types) };

        let mut game_state = initial_game_state();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, object_types, &mut game_state, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut game_state_needed = initial_game_state();
        let _ = game_state_needed.generate_object_id();
        let added_object = type2().instantiate_object(1);
        game_state_needed.objects.push(added_object);

        assert_eq!(game_state, game_state_needed);
    }

    #[test]
    fn add_one_object() {
        let instructions = vec![
            VMCommand::CallObjectAction,
            VMCommand::Halt,
            //{
            VMCommand::Function { n_locals: 0 }, // Добавляет на доску одну карту типа 2
            VMCommand::PushConstant(Word::Numeric(2)),
            VMCommand::InstanceObjectByTypeId,
            VMCommand::PushConstant(Word::Numeric(5)),
            VMCommand::Return,
            //}
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let object_types = vec![type1(), type2()];
        let object_types = unsafe { ObjectTypesRom::from_raw_parts(&object_types) };

        let mut game_state = initial_game_state();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, object_types, &mut game_state, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![5], 0, 0, 1, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut game_state_needed = initial_game_state();
        let object = type2().instantiate_object(game_state_needed.generate_object_id());
        game_state_needed.objects.push(object);

        assert_eq!(game_state, game_state_needed);
    }

    #[test]
    fn remove_one_object() {
        let instructions = vec![
            VMCommand::PushConstant(Word::Numeric(0)),
            VMCommand::RemoveObjectByIndex,
            VMCommand::Halt,
        ];
        let instructions = InstructionRom::from_vm_commands(&instructions);
        let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

        let object_types = vec![type1(), type2()];
        let object_types = unsafe { ObjectTypesRom::from_raw_parts(&object_types) };

        let mut game_state = testing_game_state();

        let args = vec![];
        let mut vm = VM::init_vm(instructions, object_types, &mut game_state, &args, 0, 0);

        vm.execute(10).unwrap();

        assert!(vm.is_halted());

        let memory = VM::release_memory(vm);
        let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2, 0, 0) };

        assert_eq!(memory, needed_memory);

        let mut game_state_needed = testing_game_state();
        game_state_needed.objects.remove(0);

        assert_eq!(game_state, game_state_needed);
    }
}
