//! State transition types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_pack::IsInitialized, pubkey::Pubkey};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct TokenMarket {
    pub is_initialized: bool,
    pub token_bank: Pubkey,
    pub tradable_token_mint: Pubkey,
    pub authority: Pubkey,
    pub accept_token_mint: Pubkey,
}

impl TokenMarket {
    pub const LEN: usize = 32*4 + 1;
}

impl IsInitialized for TokenMarket {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
