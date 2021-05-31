//! Instruction types

use crate::state::*;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_pack::Pack;
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};
#[cfg(feature = "solana-sdk")]
use solana_sdk::{
    signers::Signers,
    hash::Hash,
    rent::Rent,
    signature::{Keypair, Signer},
    system_instruction::create_account,
    transaction::Transaction,
};
use spl_token::state::{Account, Mint};

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
    owner: &Pubkey,
    market: &Pubkey,
    authority: &Pubkey,
    bank: &Pubkey,
    emitter: &Pubkey,
    acceptable: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new_readonly(*owner, false),
        AccountMeta::new(*market, false),
        AccountMeta::new_readonly(*authority, false),
        AccountMeta::new(*bank, false),
        AccountMeta::new(*emitter, false),
        AccountMeta::new_readonly(*acceptable, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Ok(Instruction::new_with_borsh(
        crate::id(),
        &TokenMarketInstructions::Initialize,
        accounts,
    ))
}

/// Create `BuyTokens` instruction
pub fn buy_tokens(
    market: &Pubkey,
    authority: &Pubkey,
    emitter: &Pubkey,
    bank: &Pubkey,
    recipient_account: &Pubkey,
    write_off_acc: &Pubkey,
    amount: u64,
) -> Result<Instruction, ProgramError> {
    let accounts = vec![
        AccountMeta::new_readonly(*market, false),
        AccountMeta::new_readonly(*authority, false),
        AccountMeta::new(*emitter, false),
        AccountMeta::new(*bank, false),
        AccountMeta::new(*recipient_account, false),
        AccountMeta::new_readonly(*write_off_acc, false),
        AccountMeta::new_readonly(spl_token::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Ok(Instruction::new_with_borsh(
        crate::id(),
        &TokenMarketInstructions::BuyTokens { amount },
        accounts,
    ))
}

#[cfg(feature = "solana-sdk")]
pub fn transaction_initialize<'s, T: Signer + ?Sized>(
    recent_blockhash: Hash,
    payer: &'s T,
    owner: &Pubkey,
    market: &'s T,
    authority: &Pubkey,
    bank: &'s T,
    emitter: &'s T,
    mint_of_acceptable: &Pubkey,
) -> Transaction
where
    Vec<&'s T>: Signers,
{
    let instructions = &[
        create_account(
            &payer.pubkey(),
            &market.pubkey(),
            Rent::default().minimum_balance(TokenMarket::LEN),
            TokenMarket::LEN as u64,
            &crate::id(),
        ),
        create_account(
            &payer.pubkey(),
            &bank.pubkey(),
            Rent::default().minimum_balance(Account::LEN),
            Account::LEN as u64,
            &spl_token::id(),
        ),
        create_account(
            &payer.pubkey(),
            &emitter.pubkey(),
            Rent::default().minimum_balance(Mint::LEN),
            Mint::LEN as u64,
            &spl_token::id(),
        ),
        initialize(
            owner,
            &market.pubkey(),
            authority,
            &bank.pubkey(),
            &emitter.pubkey(),
            mint_of_acceptable,
        )
        .unwrap(),
    ];

    let mut ts = Transaction::new_with_payer(instructions, Some(&payer.pubkey()));
    let signers = &vec![payer, market, bank, emitter];
    ts.sign(signers, recent_blockhash);

    ts
}

#[cfg(feature = "solana-sdk")]
pub fn transaction_buy_tokens<'s, T: Signer + ?Sized>(
    recent_blockhash: Hash,
    fee_payer: &'s T,
    buyer: &'s T,
    market: Pubkey,
    market_authority: Pubkey,
    emitter: Pubkey,
    bank: Pubkey,
    recipient_account: Pubkey,
    write_off_account: Pubkey,
    amount: u64,
) -> Transaction
where
    Vec<&'s T>: Signers,
{
    let instructions = &[
        spl_token::instruction::approve(
            &spl_token::id(),
            &write_off_account,
            &market_authority,
            &buyer.pubkey(),
            &[&buyer.pubkey()],
            amount,
        )
        .unwrap(),
        buy_tokens(
            &market,
            &market_authority,
            &emitter,
            &bank,
            &recipient_account,
            &write_off_account,
            amount,
        )
        .unwrap(),
    ];

    let mut ts = Transaction::new_with_payer(instructions, Some(&fee_payer.pubkey()));
    ts.sign(&vec![fee_payer, buyer], recent_blockhash);

    ts
}
