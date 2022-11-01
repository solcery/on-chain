use clap::{App, Arg};

use shellexpand::tilde;
use solana_client::rpc_client::RpcClient;
use solana_program::{
    instruction::Instruction as SolanaInstruction, program_pack::Pack, pubkey::Pubkey,
    system_instruction::create_account, system_program::ID as SystemID,
};
use solana_sdk::{
    instruction::AccountMeta, signature::Signer, signer::keypair::Keypair,
    sysvar::rent::ID as RentSysvar, transaction::Transaction,
};
use spl_token::{state::Account as TokenAccount, ID as TokenID};

use std::fs::File;

use solcery_db_program::{
    instruction::*,
    state::{GLOBAL_STATE_SEED, MINT_SEED},
};

const AMOUNT: u64 = 1_000_000_000;
fn main() -> std::io::Result<()> {
    //TODO: make mandatory arguments actually mandatory via clap tools
    let matches = App::new("Solcery DB-program management utility")
        .version(clap::crate_version!())
        .about("A small tool for managing on-chain db-program")
        .arg(
            Arg::with_name("token")
                .short("t")
                .long("token")
                .help("Path to access token keypair")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("network")
                .short("n")
                .long("network")
                .help("Solana network")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("payer")
                .short("p")
                .long("payer")
                .help("Path to access token keypair (defaults to ~/.config/solana/id.json)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("program_id")
                .long("program_id")
                .help("address of the Solcery DB program")
                .takes_value(true),
        )
        .get_matches();

    let keypair_path: String = tilde(
        matches
            .value_of("payer")
            .unwrap_or("~/.config/solana/id.json"),
    )
    .into_owned();

    println!("Reading keypair from {}", &keypair_path);

    let keypair_file = File::open(keypair_path)?;

    let keypair_array: Vec<u8> = serde_json::from_reader(keypair_file)?;

    let admin: Keypair = Keypair::from_bytes(&keypair_array).expect("Failed to parse keypair file");
    println!("Using funding pubkey: {}", admin.pubkey());

    let token_path: String = tilde(
        matches
            .value_of("token")
            .expect("Token path must be provided."),
    )
    .into_owned();

    println!("Reading token keypair from {}", &token_path);

    let token_file = File::open(token_path)?;

    let token_array: Vec<u8> = serde_json::from_reader(token_file)?;

    let token_key: Keypair = Keypair::from_bytes(&token_array).expect("Failed to parse token file");
    println!("Using token pubkey: {}", admin.pubkey());
    let token_id = token_key.pubkey();

    let program_id = matches
        .value_of("program_id")
        .expect("program_id should be provided");

    let program_id = Pubkey::try_from(program_id).unwrap();

    let (mint_id, mint_bump) = Pubkey::find_program_address(&[MINT_SEED], &program_id);
    let (global_state_id, state_bump) =
        Pubkey::find_program_address(&[GLOBAL_STATE_SEED], &program_id);

    let params = BootstrapParams {
        mint_bump,
        state_bump,
        lamports_to_global_state: AMOUNT,
        lamports_to_mint: AMOUNT,
    };

    let bootstrap_db_program = SolanaInstruction::new_with_borsh(
        program_id,
        &DBInstruction::Bootstrap(params),
        vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(mint_id, false),
            AccountMeta::new(global_state_id, false),
            AccountMeta::new(token_id, true),
            AccountMeta::new_readonly(SystemID, false),
            AccountMeta::new_readonly(TokenID, false),
            AccountMeta::new(RentSysvar, false),
        ],
    );

    let create_token_account = create_account(
        &admin.pubkey(),
        &token_id,
        AMOUNT,
        TokenAccount::get_packed_len() as u64,
        &TokenID,
    );

    let mut token_transaction = Transaction::new_with_payer(
        &[create_token_account, bootstrap_db_program],
        Some(&admin.try_pubkey().unwrap()),
    );

    let url = matches
        .value_of("network")
        .unwrap_or("https://api.devnet.solana.com");

    let client = RpcClient::new(url);
    let recent_blockhash = client.get_latest_blockhash().unwrap();

    token_transaction.sign(&[&admin, &token_key], recent_blockhash);

    let signature = client.send_transaction(&token_transaction).unwrap();

    println!("Successfuly bootstraped DB: {}", signature);

    Ok(())
}
