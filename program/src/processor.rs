//! Program state processor

use std::convert::TryFrom;

use crate::error::TokenMarketError;
use crate::instruction::TokenMarketInstructions;
use crate::state::TokenMarket;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::next_account_info,
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::{
    self,
    instruction::{initialize_account, initialize_mint, mint_to, transfer},
    solana_program::program_pack::IsInitialized,
    state::{Account, Mint},
};

/// Program state handler.
pub struct Processor;

impl Processor {
    /// Processes an instruction
    pub fn process_instruction<'accounts>(
        program_id: &Pubkey,
        accounts: &[AccountInfo<'accounts>],
        input: &[u8],
    ) -> ProgramResult {
        let instruction = TokenMarketInstructions::try_from_slice(input)?;
        let account_info_iter = &mut accounts.iter();
        match instruction {
            TokenMarketInstructions::Initialize => {
                msg!("Instruction: InitMarket");

                let owner_info = next_account_info(account_info_iter)?;
                let market_info = next_account_info(account_info_iter)?;
                let authority_market_info = next_account_info(account_info_iter)?;
                let bank_info = next_account_info(account_info_iter)?;
                let emitter_info = next_account_info(account_info_iter)?;
                let mint_of_acceptable_info = next_account_info(account_info_iter)?;
                let token_program_info = next_account_info(account_info_iter)?;
                Self::process_init_market(
                    program_id,
                    owner_info,
                    market_info,
                    authority_market_info,
                    bank_info,
                    emitter_info,
                    mint_of_acceptable_info,
                    token_program_info,
                )
            }
            TokenMarketInstructions::BuyTokens { amount } => {
                msg!("Instruction: BuyTokens");

                let token_market_info = next_account_info(account_info_iter)?;
                let authority_market_info = next_account_info(account_info_iter)?;
                let emitter_info = next_account_info(account_info_iter)?;
                let bank_info = next_account_info(account_info_iter)?;
                let recipient_account_info = next_account_info(account_info_iter)?;
                let write_off_acc_info = next_account_info(account_info_iter)?;
                let token_program_info = next_account_info(account_info_iter)?;
                Self::process_buy_tokens(
                    program_id,
                    token_market_info,
                    authority_market_info,
                    emitter_info,
                    bank_info,
                    recipient_account_info,
                    write_off_acc_info,
                    token_program_info,
                    amount,
                )
            }
        }
    }

    /// Process [InitMarket](enum.TokenMarketInstructions.html) instruction
    pub fn process_init_market<'account>(
        program_id: &Pubkey,
        owner_info: &AccountInfo,
        market_info: &AccountInfo,
        authority_market_info: &AccountInfo,
        bank_info: &AccountInfo<'account>,
        emitter_info: &AccountInfo<'account>,
        mint_of_acceptable_info: &AccountInfo<'account>,
        token_program_info: &AccountInfo<'account>,
    ) -> ProgramResult {
        let token_market = TokenMarket::try_from_slice(&market_info.data.borrow())?;
        if token_market.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }
        let mint_of_acceptable = Mint::unpack_from_slice(&mint_of_acceptable_info.data.borrow())?;

        let (key, seed) =
            Pubkey::find_program_address(&[&market_info.key.to_bytes()[..32]], program_id);
        if *authority_market_info.key != key {
            return Err(TokenMarketError::IncorrectAuthority.into());
        }

        invoke(
            &initialize_account(
                &spl_token::id(),
                &bank_info.key,
                mint_of_acceptable_info.key,
                &authority_market_info.key,
            )?,
            &[
                token_program_info.clone(),
                bank_info.clone(),
                mint_of_acceptable_info.clone(),
            ],
        )?;

        invoke(
            &initialize_mint(
                &spl_token::id(),
                emitter_info.key,
                &authority_market_info.key,
                None,
                mint_of_acceptable.decimals,
            )?,
            &[token_program_info.clone(), emitter_info.clone()],
        )?;

        TokenMarket {
            is_initialized: true,
            owner: *owner_info.key,
            bank: *bank_info.key,
            emitter_mint: *emitter_info.key,
            mint_of_acceptable: *mint_of_acceptable_info.key,
            seed,
        }
        .serialize(&mut *market_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_buy_tokens<'accounts>(
        program_id: &Pubkey,
        market_info: &AccountInfo<'accounts>,
        authority_market_info: &AccountInfo,
        emitter_info: &AccountInfo<'accounts>,
        bank_info: &AccountInfo<'accounts>,
        recipient_account_info: &AccountInfo<'accounts>,
        write_off_account_info: &AccountInfo<'accounts>,
        token_program_info: &AccountInfo<'accounts>,
        amount: u64,
    ) -> ProgramResult {
        if market_info.owner != program_id {
            return Err(ProgramError::IncorrectProgramId);
        }
        let token_market = TokenMarket::try_from_slice(*market_info.data.borrow())?;
        if !token_market.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }

        let write_off_account = Account::unpack_from_slice(*write_off_account_info.data.borrow())?;

        // check that there are enough tokens to exchange the requested number of tokens
        if write_off_account.amount < amount {
            return Err(ProgramError::InsufficientFunds);
        }

        invoke_signed(
            &transfer(
                &spl_token::id(),
                write_off_account_info.key,
                &token_market.bank,
                &authority_market_info.key,
                &[],
                amount,
            )?,
            &[
                token_program_info.clone(),
                write_off_account_info.clone(),
                bank_info.clone(),
            ],
            &[&[&market_info.key.to_bytes()[..32], &[token_market.seed]]],
        )?;

        invoke_signed(
            &mint_to(
                &spl_token::id(),
                &token_market.emitter_mint,
                &recipient_account_info.key,
                &authority_market_info.key,
                &[],
                amount,
            )?,
            &[
                token_program_info.clone(),
                emitter_info.clone(),
                market_info.clone(),
                recipient_account_info.clone(),
            ],
            &[&[&market_info.key.to_bytes()[..32], &[token_market.seed]]],
        )?;

        Ok(())
    }
}
