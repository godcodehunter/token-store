//! Program state processor

use crate::state::TokenMarket;
use crate::{error::TokenMarketError, instruction::TokenMarketInstructions};
use borsh::BorshDeserialize;
use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction,
    sysvar::Sysvar,
};
use solana_sdk::{signature::Keypair, signer::Signer};
use spl_token::{
    self,
    instruction::{initialize_account, initialize_mint, mint_to, set_authority},
    native_mint,
    solana_program::program_pack::IsInitialized,
    state::{self, Account},
};

/// Program state handler.
pub struct Processor;

impl Processor {
    // /// Calculates the authority id by generating a program address.
    // pub fn authority_id(
    //     program_id: &Pubkey,
    //     pool: &Pubkey,
    //     bump_seed: u8,
    // ) -> Result<Pubkey, ProgramError> {
    //     Pubkey::create_program_address(&[&pool.to_bytes()[..32], &[bump_seed]], program_id)
    //         .map_err(|_| PoolError::InvalidAuthorityData.into())
    // }

    /// Process [InitMarket](enum.TokenMarketInstructions.html) instruction
    pub fn process_init_market(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let owner_info = next_account_info(account_info_iter)?;
        let token_market =
            TokenMarket::try_from_slice(*next_account_info(account_info_iter)?.data.borrow())?;
        if token_market.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let bank_account = next_account_info(account_info_iter)?;
        let mint_account = next_account_info(account_info_iter)?;

        let rent = Rent::from_account_info(next_account_info(account_info_iter)?)?;

        let mint_capacity = state::Mint::get_packed_len();
        let mint_balance = rent.minimum_balance(mint_capacity);
        // let mint_authority = Self::authority_id(program_id, pool_account_info.key, pool.bump_seed)?;
        let mint_pubkey = Keypair::new().pubkey();

        let token_account = Keypair::new().pubkey();
        let token_capacity = Account::get_packed_len();
        let token_balance = rent.minimum_balance(token_capacity);

        // msg!("Create mint account...");
        // invoke(
        //     // https://github.com/solana-labs/solana-program-library/blob/0eadd438903f2f450826ac04b471075aa2cb45c4/token/cli/src/main.rs#L220:4
        //     // spl-token create-token
        //     &system_instruction::create_account(
        //         // initial balance funder
        //         owner_info.key,
        //         // address of the new account
        //         &mint_pubkey,
        //         // initial balance
        //         mint_balance,
        //         // number bytes to allocate
        //         mint_capacity as u64,
        //         // who owns that account
        //         program_id,
        //     ),
        //     &[*owner_info]
        // )?;
        // msg!("Initialize mint...");
        // invoke(
        //     &initialize_mint(
        //         &spl_token::id(),
        //         &mint_pubkey,
        //         &mint_authority,
        //         Some(&mint_authority),
        //         native_mint::DECIMALS,
        //     )?,
        //     &[]
        // );

        // // https://github.com/solana-labs/solana-program-library/blob/0eadd438903f2f450826ac04b471075aa2cb45c4/token/cli/src/main.rs#L267
        // // spl-token create-account
        // system_instruction::create_account(
        //     owner_info.key,
        //     &token_account,
        //     token_balance,
        //     token_balance as u64,
        //     program_id,
        // ),
        // initialize_account(
        //     &spl_token::id(),
        //     &token_account,
        //     &mint_pubkey,
        //     program_id,
        // )?,

        let creation_cost = mint_balance + token_balance;
        if owner_info.lamports() < creation_cost {
            return Err(ProgramError::AccountNotRentExempt);
        }

        Ok(())
    }

    pub fn process_buy_tokens(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let token_market_info = next_account_info(account_info_iter)?;
        let token_market = TokenMarket::try_from_slice(*token_market_info.data.borrow())?;
        if !token_market.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }
        let holder_info = next_account_info(account_info_iter)?;
        let holder_acc = Account::unpack_from_slice(*holder_info.data.borrow())?;
        // validate token holder account
        if !holder_acc.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }
        if holder_acc.mint != token_market.accept_token_mint {
            return Err(ProgramError::InvalidAccountData);
        }

        let holder_owner = next_account_info(account_info_iter)?;

        let recipient = next_account_info(account_info_iter)?;
        let recipient_acc = Account::unpack_from_slice(*recipient.data.borrow())?;
        // validate token recipient account
        if !recipient_acc.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }
        if recipient_acc.mint != token_market.tradable_token_mint {
            return Err(ProgramError::InvalidAccountData);
        }
        let token_program = next_account_info(account_info_iter)?;

        // check that there are enough tokens to exchange the requested number of tokens
        if holder_info.lamports() < amount {
            return Err(ProgramError::InsufficientFunds);
        }

        let (pda, _bump_seed) = Pubkey::find_program_address(&[b"token-market"], program_id);

        invoke(
            &set_authority(
                &token_program.key,
                holder_info.key,
                Some(&pda),
                spl_token::instruction::AuthorityType::AccountOwner,
                &holder_owner.key,
                &[&holder_owner.key],
            )?,
            &[
                holder_info.clone(),
                holder_owner.clone(),
                token_program.clone(),
            ],
        )?;

        invoke(
            &mint_to(
                &token_program.key,
                &token_market.tradable_token_mint,
                &recipient.key,
                &token_market.tradable_token_mint,
                &[&token_market.minting_authority],
                amount,
            )?,
            &[
                token_program.clone(),
                token_market_info.clone(),
                recipient.clone(),
            ],
        )?;

        Ok(())
    }

    /// Processes an instruction
    pub fn process_instruction(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        input: &[u8],
    ) -> ProgramResult {
        let instruction = TokenMarketInstructions::try_from_slice(input)?;
        match instruction {
            TokenMarketInstructions::Initialize => {
                msg!("Instruction: InitMarket");
                Self::process_init_market(program_id, accounts)
            }
            TokenMarketInstructions::BuyTokens { amount } => {
                msg!("Instruction: BuyTokens");
                Self::process_buy_tokens(program_id, accounts, amount)
            }
        }
    }
}
