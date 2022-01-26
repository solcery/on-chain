use super::*;
use pretty_assertions::assert_eq;

#[test]
fn correct_input() {
    let program_id = Pubkey::new_unique();
    let signer_key = Pubkey::new_unique();

    let mut signer_account_data = [0; 2]; // This is arbitrary number, just to fill AccountInfo
    let mut signer_balance = 0;

    let signer = AccountInfo::new(
        &signer_key,
        true,
        false,
        &mut signer_balance,
        &mut signer_account_data,
        &spl_token::ID,
        false,
        0,
    );
    let (pda, _bump_seed) =
        Pubkey::find_program_address(&[b"player", signer_key.as_ref()], &program_id);

    let player = Player::from_pubkey(pda.clone());

    let account_data_expected = (CURRENT_PLAYER_VERSION, player).try_to_vec().unwrap();
    let mut player_account_data = vec![0; account_data_expected.len()];
    let mut player_balance = 10;
    let player_account_info = AccountInfo::new(
        &pda,
        false,
        true,
        &mut player_balance,
        &mut player_account_data,
        &spl_token::ID,
        false,
        0,
    );
    create_player_account(&program_id, &signer, &player_account_info).unwrap();

    let account_data: &[u8] = &player_account_info.data.borrow();

    assert_eq!(account_data_expected.as_slice(), account_data);
}
