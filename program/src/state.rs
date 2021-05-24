//! State transition types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_pack::IsInitialized, pubkey::Pubkey};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct TokenMarket {
    pub is_initialized: bool,
    pub owner: Pubkey,
    pub bank: Pubkey,
    pub emitter_mint: Pubkey,
    pub authority: Pubkey,
    pub mint_of_acceptable: Pubkey,
}

impl TokenMarket {
    pub const LEN: usize = 32 * 5 + 1;
}

impl IsInitialized for TokenMarket {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
