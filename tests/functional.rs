use flexbuffers::FlexbufferSerializer;
use serde::{Deserialize, Serialize};
use solana_program::{instruction::Instruction, pubkey::Pubkey};
use solana_program_test::*;
use solana_sdk::{
    account::Account, instruction::AccountMeta, signature::Signer, transaction::Transaction,
};
use std::str::FromStr;

use solcery::{
    board::Board,
    card::{Card, CardType, EntryPoint},
    entrypoint::process_instruction,
    rom::Rom,
    vmcommand::VMCommand,
    word::Word,
    word_vec,
};

#[tokio::test]
#[ignore]
async fn initialize_dummy() {
    let program_id = Pubkey::new_unique();
    let rom_id = Pubkey::new_unique();
    let board_id = Pubkey::new_unique();
    let player_id = Pubkey::new_unique();

    let rom_account = Account::new_data(1_000, &generate_testing_rom(), &rom_id).unwrap();
    let board_account = Account::new_data(1_000, &generate_testing_board(), &board_id).unwrap();

    let mut program = ProgramTest::new("solcery", program_id, processor!(process_instruction));
    program.add_account(rom_id, rom_account);
    program.add_account(board_id, board_account);

    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let mut instruction_bytes = vec![0];
    let cardtype_index_bytes = u32::to_le_bytes(0);
    let action_index_bytes = u32::to_le_bytes(0);
    instruction_bytes.extend_from_slice(&cardtype_index_bytes);
    instruction_bytes.extend_from_slice(&action_index_bytes);

    // args vector serialization
    let mut ser = FlexbufferSerializer::new();
    Vec::<Word>::new().serialize(&mut ser).unwrap();

    instruction_bytes.extend_from_slice(ser.view());

    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_bytes(
            program_id,
            &instruction_bytes,
            vec![
                AccountMeta::new(payer.try_pubkey().unwrap(), false),
                AccountMeta::new_readonly(rom_id, false),
                AccountMeta::new(board_id, false),
            ],
        )],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}

fn generate_testing_rom() -> Rom {
    // CardType: Dummy
    // Only two actions: create dummy, remove dummy

    let dummy_attrs = word_vec![];
    let dummy_init_attrs = word_vec![];
    let entrypoints = unsafe {
        vec![
            EntryPoint::from_raw_parts(2, 0),
            EntryPoint::from_raw_parts(6, 1),
        ]
    };
    let dummy_cardtype = unsafe {
        CardType::from_raw_parts(
            1,                // CardType id
            dummy_attrs,      // CardType attributes
            dummy_init_attrs, // Initial values of Card attributes
            entrypoints,
        )
    };

    // Initial board state
    let initial_board = unsafe { Board::from_raw_parts(vec![], word_vec![], 0) };

    let instructions = vec![
        // Our calling convention requires this two instructions to be the first
        VMCommand::CallCardAction, // 0
        VMCommand::Halt,           // 1
        // Later will be all the functions

        // create_dummy
        // 0 arguments
        // EntryPoint {address: 2, n_args: 0}

        // Each function must start with the Function command
        // n_locals is the number of local variables, which that function uses
        VMCommand::Function { n_locals: 0 },       // 2
        VMCommand::PushConstant(Word::Numeric(1)), // 3
        VMCommand::InstanceCardByTypeId,           // 4
        // Each function must end with the Return command
        VMCommand::Return, // 5
        // remove_dummy
        // 1 argument: index of the dummy to be removed
        // EntryPoint {address: 6, n_args: 1}
        VMCommand::Function { n_locals: 0 }, // 6
        // Pushes index of the dummy (argument_0) to the stack
        VMCommand::PushArgument { index: 0 }, // 7
        VMCommand::RemoveCardByIndex,         // 8
        VMCommand::Return,                    // 9
    ];
    let card_types = vec![dummy_cardtype];

    unsafe { Rom::from_raw_parts(instructions, card_types, initial_board) }
}
fn generate_testing_board() -> Board {
    unsafe { Board::from_raw_parts(vec![], word_vec![], 0) }
}
