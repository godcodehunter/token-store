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
    /// 1  `[]` Fee payer
    /// 2. `[WRITE]` Market itself, it will hold all necessary info for trading.
    /// 3. `[WRITE]` Bank account that collect gotten token
    /// 4. `[]` Mint that emit token
    /// 5. `[]` Mint of that token we accept for trade
    /// 6. `[]` Token program 
    Initialize,
    /// Buy tokens
    ///
    /// 0. `[]` Tokens market
    /// 1. `[]` Bank
    /// 2. `[]` Tokens recipient
    /// 3. `[]` Write-off account 
    /// 4. `[]` The token program
    BuyTokens { amount: u64 },
}

/// Create `Example` instruction
pub fn initialize(
    program_id: &Pubkey,
    owner: &Pubkey,
    fee_payer: &Pubkey,
    market: &Pubkey,
    bank: &Pubkey,
    emitter: &Pubkey,
    acceptable: &Pubkey,
    token_program: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*owner, false),
        AccountMeta::new(*fee_payer, false),
        AccountMeta::new(*market, false),
        AccountMeta::new(*bank, false),
        AccountMeta::new(*emitter, false),
        AccountMeta::new(*acceptable, false),
        AccountMeta::new(*token_program, false),
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
    market: &Pubkey,
    bank: &Pubkey,
    recipient: &Pubkey,
    write_off_acc: &Pubkey,
    token_program: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new(*market, false),
        AccountMeta::new(*bank, false),
        AccountMeta::new(*recipient, false),
        AccountMeta::new(*write_off_acc, false),
        AccountMeta::new(*token_program, false),
    ];

    Ok(Instruction::new_with_borsh(
        *program_id,
        &TokenMarketInstructions::BuyTokens { amount },
        accounts,
    ))
}
