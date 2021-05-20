//! State transition types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{program_pack::IsInitialized, pubkey::Pubkey};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct TokenMarket {
    is_initialized: bool,
    pub tokken_bank: Pubkey,
    pub tradable_token_mint: Pubkey,
    pub minting_authority: Pubkey,
    pub accept_token_mint: Pubkey,
}

impl IsInitialized for TokenMarket {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
