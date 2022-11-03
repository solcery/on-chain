use clap::{Parser, Subcommand};
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
use std::str::FromStr;

use std::fs::File;

use solcery_db_program::{
    instruction::*,
    state::{GLOBAL_STATE_SEED, MINT_SEED},
};

#[derive(Debug, Parser)]
#[command(name = "db-client")]
#[command(about = "Solcery DB-program management utility", long_about = None, version = {clap::crate_version!()})]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Bootstrap newly deployed DB-program
    #[command(arg_required_else_help = true)]
    Bootstrap {
        #[arg(value_name = "PATH")]
        #[arg(required = true)]
        /// Path to access token keypair
        token: String,
        #[arg(required = true)]
        #[arg(value_name = "PUBKEY")]
        /// Pubkey of newly deployed db-program
        program_id: String,
        #[arg(
            value_name = "URL",
            default_value_t = {"https://api.devnet.solana.com".to_string()},
        )]
        /// URL of solana network
        network: String,
        /// Path ot keypair of the funding account
        #[arg(
            value_name = "PATH",
            default_value_t = {"~/.config/solana/id.json".to_string()},
        )]
        payer: String,
        /// Amount of lamports to be transferred in the created accounts
        #[arg(value_name = "LAMPORTS", default_value_t = 1_000_000_000)]
        amount: u64,
    },
}
fn main() -> std::io::Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Bootstrap {
            network,
            payer,
            amount,
            program_id,
            token,
        } => {
            let keypair_path: String = tilde(&payer).into_owned();

            println!("Reading keypair from {}", &keypair_path);

            let keypair_file = File::open(keypair_path)?;

            let keypair_array: Vec<u8> = serde_json::from_reader(keypair_file)?;

            let admin: Keypair =
                Keypair::from_bytes(&keypair_array).expect("Failed to parse keypair file");

            println!("Using funding pubkey: {}", admin.pubkey());

            let token_path: String = tilde(&token).into_owned();

            println!("Reading token keypair from {}", &token_path);

            let token_file = File::open(token_path)?;

            let token_array: Vec<u8> = serde_json::from_reader(token_file)?;

            let token_key: Keypair =
                Keypair::from_bytes(&token_array).expect("Failed to parse token file");

            println!("Using token pubkey: {}", admin.pubkey());
            let token_id = token_key.pubkey();

            let program_id = Pubkey::from_str(&program_id).unwrap();

            let (mint_id, mint_bump) = Pubkey::find_program_address(&[MINT_SEED], &program_id);
            let (global_state_id, state_bump) =
                Pubkey::find_program_address(&[GLOBAL_STATE_SEED], &program_id);

            let params = BootstrapParams {
                mint_bump,
                state_bump,
                lamports_to_global_state: amount,
                lamports_to_mint: amount,
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
                amount,
                TokenAccount::get_packed_len() as u64,
                &TokenID,
            );

            let mut token_transaction = Transaction::new_with_payer(
                &[create_token_account, bootstrap_db_program],
                Some(&admin.try_pubkey().unwrap()),
            );

            let client = RpcClient::new(network);
            let recent_blockhash = client.get_latest_blockhash().unwrap();

            token_transaction.sign(&[&admin, &token_key], recent_blockhash);

            let signature = client.send_transaction(&token_transaction).unwrap();

            println!("Successfuly bootstraped DB: {}", signature);

            Ok(())
        }
    }
}
