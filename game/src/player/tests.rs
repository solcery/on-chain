use super::*;
//use pretty_assertions::assert_eq;
//use Data as PlayerData;

//#[test]
//fn correct_input() {
//let program_id = Pubkey::new_unique();
//let signer_key = Pubkey::new_unique();

//let mut signer_account_data = [0; 2]; // This is arbitrary number, just to fill AccountInfo
//let mut signer_balance = 0;

//let signer = AccountInfo::new(
//&signer_key,
//true,
//false,
//&mut signer_balance,
//&mut signer_account_data,
//&spl_token::ID,
//false,
//0,
//);
//let (pda, _bump_seed) =
//Pubkey::find_program_address(&[b"player", signer_key.as_ref()], &program_id);

//let player = PlayerData::from_pubkey(pda.clone());

//let account_data_expected = (CURRENT_PLAYER_VERSION, player).try_to_vec().unwrap();
//let mut player_account_data = vec![0; account_data_expected.len()];
//let mut player_balance = 10;
//let player_account_info = AccountInfo::new(
//&pda,
//false,
//true,
//&mut player_balance,
//&mut player_account_data,
//&spl_token::ID,
//false,
//0,
//);

//let player_info = Player::new(&program_id, &signer, &player_account_info).unwrap();
//player_info.pack().unwrap();

//let account_data: &[u8] = &player_account_info.data.borrow();

//assert_eq!(account_data_expected.as_slice(), account_data);
//}

//#[test]
//fn player_account_too_small() {
//let program_id = Pubkey::new_unique();
//let signer_key = Pubkey::new_unique();

//let mut signer_account_data = [0; 2]; // This is arbitrary number, just to fill AccountInfo
//let mut signer_balance = 0;

//let signer = AccountInfo::new(
//&signer_key,
//true,
//false,
//&mut signer_balance,
//&mut signer_account_data,
//&spl_token::ID,
//false,
//0,
//);
//let (pda, _bump_seed) =
//Pubkey::find_program_address(&[b"player", signer_key.as_ref()], &program_id);

//let mut player_account_data = vec![0; 1];
//let mut player_balance = 10;
//let player_account_info = AccountInfo::new(
//&pda,
//false,
//true,
//&mut player_balance,
//&mut player_account_data,
//&spl_token::ID,
//false,
//0,
//);

//let player_info = Player::new(&program_id, &signer, &player_account_info).unwrap();
//let result = player_info.pack();

//assert_eq!(result, Err(Error::AccountTooSmall));
//}

//#[test]
//fn not_signed() {
//let program_id = Pubkey::new_unique();
//let signer_key = Pubkey::new_unique();

//let mut signer_account_data = [0; 2]; // This is arbitrary number, just to fill AccountInfo
//let mut signer_balance = 0;

//let signer = AccountInfo::new(
//&signer_key,
//false,
//false,
//&mut signer_balance,
//&mut signer_account_data,
//&spl_token::ID,
//false,
//0,
//);
//let (pda, _bump_seed) =
//Pubkey::find_program_address(&[b"player", signer_key.as_ref()], &program_id);

//let mut player_account_data = vec![0; 1];
//let mut player_balance = 10;
//let player_account_info = AccountInfo::new(
//&pda,
//false,
//true,
//&mut player_balance,
//&mut player_account_data,
//&spl_token::ID,
//false,
//0,
//);
//let result = Player::new(&program_id, &signer, &player_account_info);

//assert_eq!(result, Err(Error::NotSigned));
//}

//#[test]
//fn wrong_player_account() {
//let program_id = Pubkey::new_unique();
//let signer_key = Pubkey::new_unique();
//let player_key = Pubkey::new_unique();

//let mut signer_account_data = [0; 2]; // This is arbitrary number, just to fill AccountInfo
//let mut signer_balance = 0;

//let signer = AccountInfo::new(
//&signer_key,
//true,
//false,
//&mut signer_balance,
//&mut signer_account_data,
//&spl_token::ID,
//false,
//0,
//);

//let player = PlayerData::from_pubkey(player_key.clone());

//let account_data_expected = (CURRENT_PLAYER_VERSION, player).try_to_vec().unwrap();
//let mut player_account_data = vec![0; account_data_expected.len()];
//let mut player_balance = 10;
//let player_account_info = AccountInfo::new(
//&player_key,
//false,
//true,
//&mut player_balance,
//&mut player_account_data,
//&spl_token::ID,
//false,
//0,
//);

//let result = Player::new(&program_id, &signer, &player_account_info);

//assert_eq!(result, Err(Error::WrongPlayerAccount));
//}
