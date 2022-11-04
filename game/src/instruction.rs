use borsh::{BorshDeserialize, BorshSerialize};

use crate::state::{
    container::Container,
    game::{Game, Status as GameStatus},
    game_state::{Event, State},
};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, BorshSerialize, BorshDeserialize)]
pub enum Instruction {
    /// Fill a special [Player](Player) account for signer, where all the metainformation will be stored.
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[writable]` Player account with correct PDA
    CreatePlayerAccount,
    /// Updates [Player](Player) account from old version.
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[writable]` Player account with correct PDA
    UpdatePlayerAccount,
    /// Fill  [Game](Game) account for signer, where all the metainformation of the game will be stored.
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[]` Player account with correct PDA
    /// 2. `[]` GameProject account
    /// 3. `[writable]` Game account
    /// 4. `[writable]` GameState account
    CreateGame { num_players: u32, max_items: u32 },
    /// Add (Player)[Player] to the existing (Game)[Game].
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[writable]` Player account with correct PDA
    /// 3. `[writable]` Game account
    JoinGame,
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[writable]` Player account with correct PDA
    /// 3. `[writable]` Game account
    /// for each NFT this accounts should be provided:
    /// 1. `[]` token account
    /// 2. `[]` mint account
    AddItems { num_items: u32 },
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[]` Player account with correct PDA
    /// 3. `[writable]` Game account
    SetGameStatus { new_game_status: GameStatus },
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[]` Player account with correct PDA
    /// 3. `[writable]` Game account
    /// 4. `[]` (Optional) Event account
    AddEvent {
        event_container: Container<Vec<Event>>,
        state_step: u32,
    },
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[writable]` Player account with correct PDA
    /// 3. `[writable]` Game account
    LeaveGame,
}
