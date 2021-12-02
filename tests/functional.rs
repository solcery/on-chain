use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{instruction::Instruction, pubkey::Pubkey};
use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::AccountMeta,
    signature::Signer,
    signer::keypair::Keypair,
    transaction::Transaction,
};

use solcery_vm::{
    board::Board,
    card::{Card, CardType, EntryPoint},
    entrypoint::process_instruction,
    rom::Rom,
    vmcommand::VMCommand,
    word::Word,
    word_vec,
};

#[tokio::test]
async fn initialize_dummy() {
    let program_id = Pubkey::new_unique();
    let rom_id = Pubkey::new_unique();
    let board_id = Pubkey::new_unique();

    // This is the Pubkey of the solcery engine, which will be the only entity, who sends
    // transactions to the Solcery VM
    let engine_keypair = Keypair::new();
    let engine_id = engine_keypair.try_pubkey().unwrap();

    let rom = generate_testing_rom();
    let rom = rom.try_to_vec().unwrap();
    let mut rom_account = AccountSharedData::new(1_000, 1024, &engine_id);
    rom_account.set_data(rom);

    let board = unsafe { Board::from_raw_parts(vec![], word_vec![], 0) };
    let mut board = board.try_to_vec().unwrap();
    board.extend(std::iter::repeat(0).take(24 - board.len()));
    // Actually board should be also owned by engine, however it is impossible to test it's
    // behavior under solana-program-test (because engine_keypair should be the payer)
    let mut board_account = AccountSharedData::new(1_000, 1024, &program_id);
    board_account.set_data(board);

    let mut program = ProgramTest::new("solcery_vm", program_id, processor!(process_instruction));
    program.add_account(rom_id, Account::from(rom_account));
    program.add_account(board_id, Account::from(board_account));

    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    // Add dummy
    let mut instruction_bytes = vec![0];
    let cardtype_index_bytes = u32::to_le_bytes(0);
    let action_index_bytes = u32::to_le_bytes(0);
    instruction_bytes.extend_from_slice(&cardtype_index_bytes);
    instruction_bytes.extend_from_slice(&action_index_bytes);

    // args vector serialization
    let arg_bytes = Vec::<Word>::new().try_to_vec().unwrap();
    instruction_bytes.extend_from_slice(&arg_bytes);

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
        Some(&payer.try_pubkey().unwrap()),
    );

    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
    let new_board_account = banks_client.get_account(board_id).await.unwrap().unwrap();
    let board_data = new_board_account.data;
    let new_board: Board = BorshDeserialize::deserialize(&mut board_data.as_slice()).unwrap();

    // Composing expected result
    let dummy_card = unsafe { Card::from_raw_parts(0, 1, word_vec![]) };
    let expected_board = unsafe { Board::from_raw_parts(vec![dummy_card], word_vec![], 1) };

    assert_eq!(new_board, expected_board);
}

#[tokio::test]
async fn remove_dummy() {
    let program_id = Pubkey::new_unique();
    let rom_id = Pubkey::new_unique();
    let board_id = Pubkey::new_unique();

    // This is the Pubkey of the solcery engine, which will be the only entity, who sends
    // transactions to the Solcery VM
    let engine_keypair = Keypair::new();
    let engine_id = engine_keypair.try_pubkey().unwrap();

    let rom = generate_testing_rom();
    let rom = rom.try_to_vec().unwrap();
    let mut rom_account = AccountSharedData::new(1_000, 1024, &engine_id);
    rom_account.set_data(rom);

    let dummy_card = unsafe { Card::from_raw_parts(0, 1, word_vec![]) };
    let board = unsafe { Board::from_raw_parts(vec![dummy_card], word_vec![], 1) };
    let mut board = board.try_to_vec().unwrap();
    board.extend(std::iter::repeat(0).take(24 - board.len()));
    board.len();
    // Actually board should be also owned by engine, however it is impossible to test it's
    // behavior under solana-program-test (because engine_keypair should be the payer)
    let mut board_account = AccountSharedData::new(1_000, 1024, &program_id);
    board_account.set_data(board);

    let mut program = ProgramTest::new("solcery_vm", program_id, processor!(process_instruction));
    program.add_account(rom_id, Account::from(rom_account));
    program.add_account(board_id, Account::from(board_account));

    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    // Add dummy
    let mut instruction_bytes = vec![0];
    let cardtype_index_bytes = u32::to_le_bytes(0);
    let action_index_bytes = u32::to_le_bytes(1);
    instruction_bytes.extend_from_slice(&cardtype_index_bytes);
    instruction_bytes.extend_from_slice(&action_index_bytes);

    // args vector serialization
    let arg_bytes = word_vec![0].try_to_vec().unwrap();
    instruction_bytes.extend_from_slice(&arg_bytes);

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
        Some(&payer.try_pubkey().unwrap()),
    );

    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
    let new_board_account = banks_client.get_account(board_id).await.unwrap().unwrap();
    let board_data = new_board_account.data;
    let new_board: Board = BorshDeserialize::deserialize(&mut board_data.as_slice()).unwrap();

    // Composing expected result
    let expected_board = unsafe { Board::from_raw_parts(vec![], word_vec![], 1) };

    assert_eq!(new_board, expected_board);
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
        // Each function must end with the Return or ReturnVoid command
        VMCommand::ReturnVoid, // 5
        // remove_dummy
        // 1 argument: index of the dummy to be removed
        // EntryPoint {address: 6, n_args: 1}
        VMCommand::Function { n_locals: 0 }, // 6
        // Pushes index of the dummy (argument_0) to the stack
        VMCommand::PushArgument { index: 0 }, // 7
        VMCommand::RemoveCardByIndex,         // 8
        VMCommand::ReturnVoid,                // 9
    ];
    let card_types = vec![dummy_cardtype];

    unsafe { Rom::from_raw_parts(instructions, card_types, initial_board) }
}
