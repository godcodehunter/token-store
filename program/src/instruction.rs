//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Instruction definition
#[derive(BorshSerialize, BorshDeserialize, PartialEq, Debug, Clone)]
pub enum TokenMarketInstructions {
    /// Initialize the market
    ///
    /// Accounts expected:
    ///
    /// 0. `[]` Person that own token market.
    /// 1. `[WRITE]` Market itself, it will hold all necessary info for trading.
    /// 2. `[WRITE]` Bank account that collect gotten token
    /// 3. `[]` Mint that emit token
    /// 4. `[]` Mint of that token we accept for trade
    /// 5. `[]` Rent sysvar
    Initialize,
    /// Buy tokens
    ///
    /// 0. `[]` Tokens market
    /// 1. `[]` Tokens holder - token account with tokens for that trade
    /// 2. `[]` Token holder owner
    /// 3. `[]` Tokens recipient
    /// 4. `[]` The token program
    BuyTokens { amount: u64 },
}

/// Create `Example` instruction
pub fn initialize(
    program_id: &Pubkey,
    market_account: &Pubkey,
    rent: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![AccountMeta::new(*market_account, false)];

    Ok(Instruction::new_with_borsh(
        *program_id,
        &TokenMarketInstructions::Initialize,
        accounts,
    ))
}

/// Create `BuyTokens` instruction
pub fn buy_tokens(
    program_id: &Pubkey,
    tokens_market: &Pubkey,
    tokens_holder: &Pubkey,
    tokens_recipient: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*tokens_market, false),
        AccountMeta::new(*tokens_holder, false),
        AccountMeta::new(*tokens_recipient, false),
    ];

    Ok(Instruction::new_with_borsh(
        *program_id,
        &TokenMarketInstructions::BuyTokens { amount },
        accounts,
    ))
}
