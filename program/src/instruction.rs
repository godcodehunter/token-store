//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar
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
    /// 2. `[]` Market authority
    /// 3. `[WRITE]` Bank account that collect gotten token
    /// 4. `[WRITE]` Mint that emit token
    /// 5. `[]` Mint of that token we accept for trade
    /// 6. `[]` Token program 
    /// 7. `[]` Rent sysvar
    Initialize,
    /// Buy tokens
    ///
    /// 0. `[]` Tokens market
    /// 1. `[]` Market authority
    /// 2. `[WRITE]` Emitter mint 
    /// 3. `[WRITE]` Bank
    /// 4. `[WRITE]` Tokens recipient account
    /// 5. `[]` Write-off account 
    /// 6. `[]` The token program
    /// 7. `[]` Rent sysvar
    BuyTokens { amount: u64 },
}

/// Create `Example` instruction
pub fn initialize(
    program_id: &Pubkey,
    owner: &Pubkey,
    market: &Pubkey,
    authority: &Pubkey,
    bank: &Pubkey,
    emitter: &Pubkey,
    acceptable: &Pubkey,
    token_program: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new_readonly(*owner, false),
        AccountMeta::new(*market, false),
        AccountMeta::new_readonly(*authority, false),
        AccountMeta::new(*bank, false),
        AccountMeta::new(*emitter, false),
        AccountMeta::new_readonly(*acceptable, false),
        AccountMeta::new_readonly(*token_program, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
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
    authority: &Pubkey,
    emitter:  &Pubkey,
    bank: &Pubkey,
    recipient_account: &Pubkey,
    write_off_acc: &Pubkey,
    token_program: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(*authority, false),
        AccountMeta::new(*emitter, false),
        AccountMeta::new(*bank, false),
        AccountMeta::new(*recipient_account, false),
        AccountMeta::new_readonly(*write_off_acc, false),
        AccountMeta::new_readonly(*token_program, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Ok(Instruction::new_with_borsh(
        *program_id,
        &TokenMarketInstructions::BuyTokens { amount },
        accounts,
    ))
}
