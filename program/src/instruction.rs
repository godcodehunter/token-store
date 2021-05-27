//! Instruction types

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar
};
use crate::state::*;
use solana_program::program_pack::Pack;
use spl_token::state::{Account, Mint};
#[cfg(feature = "solana-sdk")]
use solana_sdk::{
    hash::Hash,
    rent::Rent,
    transaction::Transaction,
    signature::{Keypair, Signer},
    system_instruction::create_account,
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

#[cfg(feature = "solana-sdk")]
pub fn transaction_initialize(
    recent_blockhash: Hash,
    payer: &Keypair, 
    owner: &Pubkey, 
    market: &Keypair,
    authority: &Pubkey,
    bank: &Keypair,
    emitter: &Keypair,
    mint_of_acceptable: &Pubkey,
) -> Transaction {
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
            &crate::id(),
            owner,
            &market.pubkey(),
            authority,
            &bank.pubkey(),
            &emitter.pubkey(),
            mint_of_acceptable,
            &spl_token::id(),
        ).unwrap(),
    ];

    let mut ts = Transaction::new_with_payer(
        instructions, 
        Some(&payer.pubkey())
    );
    let signers = &vec![
        payer as &dyn Signer,
        market as &dyn Signer,
        bank as &dyn Signer,
        emitter as &dyn Signer,
    ];
    ts.sign(signers, recent_blockhash);
    
    ts
}

#[cfg(feature = "solana-sdk")]
pub fn transaction_buy_tokens(
    recent_blockhash: Hash,
    fee_payer: Keypair,
    buyer: Keypair,
    market: Pubkey, 
    market_authority: Pubkey, 
    emitter: Pubkey, 
    bank: Pubkey,
    recipient_account: Pubkey,
    write_off_account: Pubkey,
    amount: u64,
) -> Transaction {
    let instructions = &[
        spl_token::instruction::approve(
            &spl_token::id(),
            &write_off_account,
            &market_authority,
            &buyer.pubkey(),
            &[&buyer.pubkey()],
            amount,
        ).unwrap(),
        buy_tokens(
            &crate::id(),
            &market,
            &market_authority,
            &emitter,
            &bank,
            &recipient_account,
            &write_off_account,
            &spl_token::id(),
            amount,
        ).unwrap(),
    ];

    let mut ts = Transaction::new_with_payer(
        instructions, 
        Some(&fee_payer.pubkey())
    );
    ts.sign(&vec![&fee_payer, &buyer], recent_blockhash);

    ts
}

