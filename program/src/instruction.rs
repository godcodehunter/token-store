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
    Initialize,
    /// Buy tokens
    ///
    /// 0. `[]` Tokens market
    /// 1. `[]` Bank
    /// 2. `[]` Tokens holder - token account with tokens for that trade
    /// 3. `[]` Token holder owner
    /// 4. `[]` Tokens recipient
    /// 5. `[]` The token program
    BuyTokens { amount: u64 },
}

/// Create `Example` instruction
pub fn initialize(
    program_id: &Pubkey,
    owner: &Pubkey,
    market_account: &Pubkey,
    bank: &Pubkey,
    emitter: &Pubkey,
    acceptable_token: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*owner, false),
        AccountMeta::new(*market_account, false),
        AccountMeta::new(*bank, false),
        AccountMeta::new(*emitter, false),
        AccountMeta::new(*acceptable_token, false),
    ];

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
