#![cfg(feature = "test-bpf")]

use solana_program::{
    pubkey::Pubkey,
    program_pack::Pack,
};
use token_market::{*, state::*, processor::*};
use spl_token::state::{Account, Mint};
use solana_program_test::*;
use solana_sdk::{
    transaction::Transaction,
    rent::Rent,
    signature::{Keypair, Signer},
    system_instruction::create_account,
};

pub fn program_test() -> ProgramTest {
    ProgramTest::new(
        "token_market",
        id(),
        processor!(Processor::process_instruction),
    )
}

#[tokio::test]
async fn test_create_market() {
    let (mut banks_client, payer, recent_blockhash) = program_test().start().await;

    let owner = Keypair::new();
    let mint_acceptable = Keypair::new();
    let market = Keypair::new();
    let bank = Keypair::new();
    let emitter = Keypair::new();

    let instructions = &[
        create_account(
            &payer.pubkey(),
            &market.pubkey(),
            Rent::default().minimum_balance(TokenMarket::LEN),
            TokenMarket::LEN as u64,
            &token_market::id(),
        ),
        create_account(
            &payer.pubkey(),
            &bank.pubkey(),
            Rent::default().minimum_balance(Account::LEN),
            Account::LEN as u64,
            &token_market::id(),
        ),
        create_account(
            &payer.pubkey(),
            &emitter.pubkey(),
            Rent::default().minimum_balance(Mint::LEN),
            Mint::LEN as u64,
            &token_market::id(),
        ),
        instruction::initialize(
            &token_market::id(),
            &owner.pubkey(),
            &market.pubkey(),
            &bank.pubkey(),
            &emitter.pubkey(),
            &emitter.pubkey(),
            &mint_acceptable.pubkey(),
            &spl_token::id(),
        ).unwrap(),
    ];

    let mut ts = Transaction::new_with_payer(
        instructions, 
        Some(&payer.pubkey())
    );

    let signers = vec![
        &payer, 
        &owner, 
        &market,
    ];

    ts.sign(&signers, recent_blockhash);
    banks_client.process_transaction(ts).await.unwrap();
}
