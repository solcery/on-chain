use super::*;
use crate::board::MemoryRegion;
use crate::card::{CardType, EntryPoint};
use crate::word_vec;
use pretty_assertions::assert_eq;

fn type1() -> CardType {
    let type1_attrs = word_vec![10, 5, true, false,];
    let type1_init_attrs = word_vec![5, 5, false, false,];
    unsafe {
        CardType::from_raw_parts(
            1,
            type1_attrs,
            type1_init_attrs,
            vec![EntryPoint::from_raw_parts(2, 0)],
        )
    }
}

fn type2() -> CardType {
    let type2_attrs = word_vec![20, 5, true, true,];
    let type2_init_attrs = word_vec![6, 4, false, false,];
    unsafe {
        CardType::from_raw_parts(
            2,
            type2_attrs,
            type2_init_attrs,
            vec![EntryPoint::from_raw_parts(4, 0)],
        )
    }
}

fn testing_board() -> Board {
    let type1 = type1();
    let type2 = type2();

    let board_attrs = word_vec![3, 4, 5, false, false, true,];

    let mut card1 = type1.instantiate_card(1);
    let mut card2 = type2.instantiate_card(2);

    card1.attrs[0] = Word::Numeric(4);
    card2.attrs[3] = Word::Boolean(true);

    let card3 = type1.instantiate_card(3);
    let card4 = type2.instantiate_card(4);
    let common_region = MemoryRegion::with_data(0, vec![card1, card2, card3, card4], board_attrs);

    unsafe { Board::from_raw_parts(vec![common_region], 5) }
}

fn initial_board() -> Board {
    let card1 = type1().instantiate_card(1);
    let board_attrs = word_vec![0, 0, 0, false, false, false,];
    let common_region = MemoryRegion::with_data(0, vec![card1], board_attrs);
    unsafe { Board::from_raw_parts(vec![common_region], 1) }
}

#[test]
fn init_empty_memory_vm() {
    let instructions = vec![
        VMCommand::PushConstant(Word::Numeric(2)),
        VMCommand::PushCardTypeByCardIndex { region_index: 0 },
        VMCommand::Halt,
    ];
    let instructions = InstructionRom::from_vm_commands(&instructions);
    let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

    let card_types = vec![type1(), type2()];
    let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

    let mut board = testing_board();

    let args = vec![];
    let vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);
    let memory = VM::release_memory(vm);
    let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 0, 0, 0) };

    assert_eq!(memory, needed_memory);
}

#[test]
#[ignore]
fn push_type() {
    let instructions = vec![
        VMCommand::PushConstant(Word::Numeric(2)),
        VMCommand::PushCardTypeByCardIndex { region_index: 0 },
        VMCommand::Halt,
    ];
    let instructions = InstructionRom::from_vm_commands(&instructions);
    let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

    let card_types = vec![type1(), type2()];
    let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

    let mut board = testing_board();

    let args = vec![];
    let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

    vm.execute(10).unwrap();
    assert!(vm.is_halted());

    let memory = VM::release_memory(vm);
    let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, 1], 0, 0, 2, 0, 0) };

    assert_eq!(memory, needed_memory);

    let board_needed = testing_board();

    assert_eq!(board, board_needed);
}

#[test]
#[ignore]
fn push_card_count() {
    let instructions = vec![
        VMCommand::PushConstant(Word::Numeric(2)),
        VMCommand::PushCardCountWithCardType { region_index: 0 },
        VMCommand::Halt,
    ];
    let instructions = InstructionRom::from_vm_commands(&instructions);
    let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

    let card_types = vec![type1(), type2()];
    let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

    let mut board = testing_board();

    let args = vec![];
    let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

    vm.execute(10).unwrap();

    assert!(vm.is_halted());

    let memory = VM::release_memory(vm);
    let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, 2], 0, 0, 2, 0, 0) };

    assert_eq!(memory, needed_memory);

    let board_needed = testing_board();

    assert_eq!(board, board_needed);
}

#[test]
fn push_type_attr_by_type_index() {
    let instructions = vec![
        VMCommand::PushConstant(Word::Numeric(1)),
        VMCommand::LoadCardTypeAttrByTypeIndex { attr_index: 3 },
        VMCommand::Halt,
    ];
    let instructions = InstructionRom::from_vm_commands(&instructions);
    let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

    let card_types = vec![type1(), type2()];
    let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

    let mut board = testing_board();

    let args = vec![];
    let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

    vm.execute(10).unwrap();

    assert!(vm.is_halted());

    let memory = VM::release_memory(vm);
    let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2, 0, 0) };

    assert_eq!(memory, needed_memory);

    let board_needed = testing_board();

    assert_eq!(board, board_needed);
}

#[test]
#[ignore]
fn push_type_attr_by_card_index() {
    let instructions = vec![
        VMCommand::PushConstant(Word::Numeric(1)),
        VMCommand::LoadCardTypeAttrByCardIndex {
            region_index: 0,
            attr_index: 3,
        },
        VMCommand::Halt,
    ];
    let instructions = InstructionRom::from_vm_commands(&instructions);
    let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

    let card_types = vec![type1(), type2()];
    let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

    let mut board = testing_board();

    let args = vec![];
    let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

    vm.execute(10).unwrap();

    assert!(vm.is_halted());

    let memory = VM::release_memory(vm);
    let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2, 0, 0) };

    assert_eq!(memory, needed_memory);

    let board_needed = testing_board();

    assert_eq!(board, board_needed);
}

#[test]
#[ignore]
fn push_attr() {
    let instructions = vec![
        VMCommand::PushConstant(Word::Numeric(1)),
        VMCommand::LoadCardAttrByCardIndex {
            region_index: 0,
            attr_index: 3,
        },
        VMCommand::Halt,
    ];
    let instructions = InstructionRom::from_vm_commands(&instructions);
    let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

    let card_types = vec![type1(), type2()];
    let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

    let mut board = testing_board();

    let args = vec![];
    let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

    vm.execute(10).unwrap();

    assert!(vm.is_halted());

    let memory = VM::release_memory(vm);
    let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0, true], 0, 0, 2, 0, 0) };

    assert_eq!(memory, needed_memory);

    let board_needed = testing_board();

    assert_eq!(board, board_needed);
}

#[test]
#[ignore]
fn pop_attr() {
    let instructions = vec![
        VMCommand::PushConstant(Word::Numeric(42)),
        VMCommand::PushConstant(Word::Numeric(1)),
        VMCommand::StoreCardAttrByCardIndex {
            region_index: 0,
            attr_index: 3,
        },
        VMCommand::Halt,
    ];
    let instructions = InstructionRom::from_vm_commands(&instructions);
    let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

    let card_types = vec![type1(), type2()];
    let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

    let mut board = testing_board();

    let args = vec![];
    let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

    vm.execute(10).unwrap();

    assert!(vm.is_halted());

    let memory = VM::release_memory(vm);
    let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 3, 0, 0) };

    assert_eq!(memory, needed_memory);

    let mut board_needed = testing_board();
    board_needed.cards[1].attrs[3] = Word::Numeric(42);

    assert_eq!(board, board_needed);
}

#[test]
#[ignore]
fn add_one_card_by_index() {
    let instructions = vec![
        VMCommand::PushConstant(Word::Numeric(1)),
        VMCommand::InstanceCardByTypeIndex { region_index: 0 },
        VMCommand::Halt,
    ];
    let instructions = InstructionRom::from_vm_commands(&instructions);
    let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

    let card_types = vec![type1(), type2()];
    let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

    let mut board = initial_board();

    let args = vec![];
    let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

    vm.execute(10).unwrap();

    assert!(vm.is_halted());

    let memory = VM::release_memory(vm);
    let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2, 0, 0) };

    assert_eq!(memory, needed_memory);

    let mut board_needed = initial_board();
    let _ = board_needed.generate_card_id();
    let added_card = type2().instantiate_card(1);
    board_needed.regions[0].cards.push(added_card);

    assert_eq!(board, board_needed);
}

#[test]
#[ignore]
fn add_one_card_by_id() {
    let instructions = vec![
        VMCommand::PushConstant(Word::Numeric(2)),
        VMCommand::InstanceCardByTypeId { region_index: 0 },
        VMCommand::Halt,
    ];
    let instructions = InstructionRom::from_vm_commands(&instructions);
    let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

    let card_types = vec![type1(), type2()];
    let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

    let mut board = initial_board();

    let args = vec![];
    let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

    vm.execute(10).unwrap();

    assert!(vm.is_halted());

    let memory = VM::release_memory(vm);
    let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2, 0, 0) };

    assert_eq!(memory, needed_memory);

    let mut board_needed = initial_board();
    let _ = board_needed.generate_card_id();
    let added_card = type2().instantiate_card(1);
    board_needed.regions[0].cards.push(added_card);

    assert_eq!(board, board_needed);
}

#[test]
#[ignore]
fn add_one_card() {
    let instructions = vec![
        VMCommand::CallCardAction,
        VMCommand::Halt,
        //{
        VMCommand::Function { n_locals: 0 }, // Добавляет на доску одну карту типа 2
        VMCommand::PushConstant(Word::Numeric(2)),
        VMCommand::InstanceCardByTypeId { region_index: 0 },
        VMCommand::PushConstant(Word::Numeric(5)),
        VMCommand::Return,
        //}
    ];
    let instructions = InstructionRom::from_vm_commands(&instructions);
    let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

    let card_types = vec![type1(), type2()];
    let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

    let mut board = initial_board();

    let args = vec![];
    let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

    vm.execute(10).unwrap();

    assert!(vm.is_halted());

    let memory = VM::release_memory(vm);
    let needed_memory = unsafe { Memory::from_raw_parts(word_vec![5], 0, 0, 1, 0, 0) };

    assert_eq!(memory, needed_memory);

    let mut board_needed = initial_board();
    let card = type2().instantiate_card(board_needed.generate_card_id());
    board_needed.regions[0].cards.push(card);

    assert_eq!(board, board_needed);
}

#[test]
#[ignore]
fn remove_one_card() {
    let instructions = vec![
        VMCommand::PushConstant(Word::Numeric(0)),
        VMCommand::RemoveCardByIndex { region_index: 0 },
        VMCommand::Halt,
    ];
    let instructions = InstructionRom::from_vm_commands(&instructions);
    let instructions = unsafe { InstructionRom::from_raw_parts(&instructions) };

    let card_types = vec![type1(), type2()];
    let card_types = unsafe { CardTypesRom::from_raw_parts(&card_types) };

    let mut board = testing_board();

    let args = vec![];
    let mut vm = VM::init_vm(instructions, card_types, &mut board, &args, 0, 0);

    vm.execute(10).unwrap();

    assert!(vm.is_halted());

    let memory = VM::release_memory(vm);
    let needed_memory = unsafe { Memory::from_raw_parts(word_vec![0, 0], 0, 0, 2, 0, 0) };

    assert_eq!(memory, needed_memory);

    let mut board_needed = testing_board();
    board_needed.regions[0].cards.remove(0);

    assert_eq!(board, board_needed);
}
