#![cfg(feature = "test-bpf")]
#![feature(slice_pattern)]
use core::slice::SlicePattern;

use solana_program::{
    system_program,
    program_pack::Pack,
};
use token_market::{
    id,
    instruction::{
        transaction_initialize,
        transaction_buy_tokens,
    },
    processor::Processor,
};
use spl_token::state::{Account, Mint};
use solana_program_test::*;
use solana_sdk::{
    hash::Hash,
    pubkey::Pubkey,
    transaction::Transaction,
    rent::Rent,
    signature::{Keypair, Signer},
    system_instruction::create_account,
};
use spl_token::instruction::{initialize_mint, mint_to, initialize_account};

fn program_test() -> ProgramTest {
    ProgramTest::new(
        "token_market",
        id(),
        processor!(Processor::process_instruction),
    )
}

/// Create owner market account and acceptable token mint 
fn preparation_test_create_market(
    recent_blockhash: Hash,
    payer: &Keypair, 
    owner: &Keypair,
    mint_of_acceptable: &Keypair,
) -> Transaction {
    let instructions = &[
        create_account(
            &payer.pubkey(),
            &owner.pubkey(),
            Rent::default().minimum_balance(0),
            0,
            &system_program::id(),
        ),
        create_account(
            &payer.pubkey(),
            &mint_of_acceptable.pubkey(),
            Rent::default().minimum_balance(Mint::LEN),
            Mint::LEN as u64,
            &spl_token::id(),
        ),
        initialize_mint(
            &spl_token::id(),
            &mint_of_acceptable.pubkey(),
            &owner.pubkey(),
            Some(&owner.pubkey()),
            9,
        ).unwrap(),
    ];

    let mut ts = Transaction::new_with_payer(
        instructions.as_ref(), 
        Some(&payer.pubkey())
    );
    let signers = &vec![
        payer as &dyn Signer, 
        owner as &dyn Signer, 
        mint_of_acceptable as &dyn Signer,
    ];
    ts.sign(signers, recent_blockhash);
    ts
}

#[tokio::test]
async fn test_create_market() {
    let pt = program_test();
    let (mut banks_client, payer, recent_blockhash) = pt.start().await;

    let owner = Keypair::new();
    let market = Keypair::new();
    let market_authority = Pubkey::find_program_address(
        &[&market.pubkey().to_bytes()[..32]],
        &token_market::id(),
    ).0;
    let bank = Keypair::new();
    let emitter = Keypair::new();
    let mint_of_acceptable = Keypair::new();

    let ts = preparation_test_create_market(
        recent_blockhash,
        &payer,
        &owner,
        &mint_of_acceptable,
    );
    banks_client.process_transaction(ts).await.unwrap();
    
    let ts = transaction_initialize(
        recent_blockhash,
        &payer, 
        &owner.pubkey(), 
        &market,
        &market_authority,
        &bank,
        &emitter,
        &mint_of_acceptable.pubkey(), 
    );
    banks_client.process_transaction(ts).await.unwrap();
}

fn preparation_test_buy_tokens(
    recent_blockhash: Hash,
    owner: &Keypair,
    payer: &Keypair,
    buyer: &Keypair, 
    recipient: &Keypair,
    mint_of_acceptable: &Pubkey,
    emitter: &Pubkey,
    write_off_account: &Keypair,
    recipient_account: &Keypair,
) -> Transaction {
    let instructions = &[
        create_account(
            &payer.pubkey(),
            &buyer.pubkey(),
            Rent::default().minimum_balance(0),
            0,
            &system_program::id(),
        ),
        create_account(
            &payer.pubkey(),
            &recipient.pubkey(),
            Rent::default().minimum_balance(0),
            0,
            &system_program::id(),
        ),
        create_account(
            &payer.pubkey(),
            &write_off_account.pubkey(),
            Rent::default().minimum_balance(Account::LEN),
            Account::LEN as u64,
            &spl_token::id(),
        ),
        initialize_account(
            &spl_token::id(),
            &write_off_account.pubkey(),
            mint_of_acceptable,
            &buyer.pubkey(),
        ).unwrap(),
        mint_to(
            &spl_token::id(),
            mint_of_acceptable,
            &write_off_account.pubkey(),
            &owner.pubkey(),
            &[],
            100,
        ).unwrap(),
        create_account(
            &payer.pubkey(),
            &recipient_account.pubkey(),
            Rent::default().minimum_balance(Account::LEN),
            Account::LEN as u64,
            &spl_token::id(),
        ),
        initialize_account(
            &spl_token::id(),
            &recipient_account.pubkey(),
            emitter,
            &recipient.pubkey(),
        ).unwrap(),
    ];

    let mut ts = Transaction::new_with_payer(
        instructions.as_ref(), 
        Some(&payer.pubkey())
    );

    let signers = vec![
        payer as &dyn Signer, 
        buyer as &dyn Signer, 
        recipient as &dyn Signer, 
        write_off_account as &dyn Signer, 
        recipient_account as &dyn Signer, 
        owner as &dyn Signer, 
    ];
    ts.sign(&signers, recent_blockhash);
    
    ts
}

#[tokio::test]
async fn test_buy_tokens() {
    let pt = program_test();
    let (mut banks_client, payer, recent_blockhash) = pt.start().await;
    
    let owner = Keypair::new();
    let market = Keypair::new();
    let market_authority = Pubkey::find_program_address(
        &[&market.pubkey().to_bytes()[..32]], 
        &token_market::id(),
    ).0;
    let buyer = Keypair::new();
    let recipient = Keypair::new();
    let emitter = Keypair::new();
    let bank = Keypair::new();
    let mint_of_acceptable = Keypair::new();
    let write_off_account = Keypair::new();
    let recipient_account = Keypair::new();
    let amount = 70;

    let ts = preparation_test_create_market(
        recent_blockhash,
        &payer,
        &owner,
        &mint_of_acceptable,
    );
    banks_client.process_transaction(ts).await.unwrap();
    
    let ts = transaction_initialize(
        recent_blockhash,
        &payer, 
        &owner.pubkey(), 
        &market,
        &market_authority,
        &bank,
        &emitter,
        &mint_of_acceptable.pubkey(), 
    );
    banks_client.process_transaction(ts).await.unwrap();

    let ts = preparation_test_buy_tokens(
        recent_blockhash,
        &owner,
        &payer, 
        &buyer, 
        &recipient, 
        &mint_of_acceptable.pubkey(), 
        &emitter.pubkey(),
        &write_off_account,
        &recipient_account,
    );
    banks_client.process_transaction(ts).await.unwrap();

    let ts = transaction_buy_tokens(
        recent_blockhash,
        payer,
        buyer,
        market.pubkey(), 
        market_authority,
        emitter.pubkey(),
        bank.pubkey(),
        recipient_account.pubkey(),
        write_off_account.pubkey(),
        amount,
    );
    banks_client.process_transaction(ts).await.unwrap();

    let b = banks_client.get_account(write_off_account.pubkey()).await.unwrap().unwrap();
    let acc = Account::unpack_from_slice(&b.data.as_slice()).unwrap();
    assert!(acc.amount == 30, "Leftover funds are not correct");
    
    let b = banks_client.get_account(bank.pubkey()).await.unwrap().unwrap();
    let acc = Account::unpack_from_slice(&b.data.as_slice()).unwrap();
    assert!(acc.amount == 70, "The amount of funds is not correct");

    let b = banks_client.get_account(recipient_account.pubkey()).await.unwrap().unwrap();
    let acc = Account::unpack_from_slice(&b.data.as_slice()).unwrap();
    assert!(acc.amount == 70, "The amount of funds is not correct");
}