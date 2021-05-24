//! Program state processor

use crate::instruction::TokenMarketInstructions;
use crate::state::TokenMarket;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::next_account_info, account_info::AccountInfo, entrypoint::ProgramResult, msg,
    program::invoke, program_error::ProgramError, program_pack::Pack, pubkey::Pubkey,
};
use spl_token::{
    self,
    instruction::{initialize_account, initialize_mint, mint_to, set_authority, transfer},
    solana_program::program_pack::IsInitialized,
    state::Account,
};

/// Program state handler.
pub struct Processor;

impl Processor {
    /// Processes an instruction
    pub fn process_instruction<'accounts>(
        program_id: &Pubkey,
        accounts: &'accounts [AccountInfo<'accounts>],
        input: &[u8],
    ) -> ProgramResult {
        let instruction = TokenMarketInstructions::try_from_slice(input)?;
        let account_info_iter = &mut accounts.iter();
        match instruction {
            TokenMarketInstructions::Initialize => {
                msg!("Instruction: InitMarket");
                
                let owner_info = next_account_info(account_info_iter)?;
                let fee_payer_info = next_account_info(account_info_iter)?;
                let market_info = next_account_info(account_info_iter)?;
                let bank_info = next_account_info(account_info_iter)?;
                let emitter_info = next_account_info(account_info_iter)?;
                let accepted_info = next_account_info(account_info_iter)?;
                let token_program_info = next_account_info(account_info_iter)?;
                Self::process_init_market(
                    program_id,
                    owner_info,
                    fee_payer_info,
                    market_info,
                    bank_info,
                    emitter_info,
                    accepted_info,
                    token_program_info,
                )
            }
            TokenMarketInstructions::BuyTokens { amount } => {
                msg!("Instruction: BuyTokens");

                let token_market_info = next_account_info(account_info_iter)?;
                let bank_info = next_account_info(account_info_iter)?;
                let recipient_info = next_account_info(account_info_iter)?;
                let write_off_acc_info = next_account_info(account_info_iter)?;
                let token_program = next_account_info(account_info_iter)?;
                Self::process_buy_tokens(
                    program_id,
                    token_market_info,
                    bank_info,
                    recipient_info,
                    write_off_acc_info,
                    token_program,
                    amount,
                )
            }
        }
    }

    /// Process [InitMarket](enum.TokenMarketInstructions.html) instruction
    pub fn process_init_market(
        program_id: &Pubkey,
        owner_info: &AccountInfo,
        fee_payer_info: &AccountInfo,
        market_info: &AccountInfo,
        bank_info: &AccountInfo,
        emitter_info: &AccountInfo,
        accepted_mint_info: &AccountInfo,
        token_program_info: &AccountInfo,
    ) -> ProgramResult {
        let token_market = TokenMarket::try_from_slice(&market_info.data.borrow())?;
        if token_market.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        let authority = Pubkey::find_program_address(&[b"token-market"], program_id).0;

        invoke(
            &initialize_account(
                token_program_info.key,
                &bank_info.key,
                accepted_mint_info.key,
                &authority,
            )?,
            &[]
        )?;

        invoke(
            &initialize_mint(
                token_program_info.key,
                emitter_info.key,
                &authority,
                Some(&authority),
                todo!(),
            )?,
            &[]
        )?;


        TokenMarket {
            is_initialized: true,
            owner: *owner_info.key,
            bank: *bank_info.key,
            emitter_mint: *emitter_info.key,
            authority: authority,
            mint_of_acceptable: *accepted_mint_info.key,
        }
        .serialize(&mut *market_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_buy_tokens<'accounts>(
        program_id: &Pubkey,
        market_info: &'accounts AccountInfo<'accounts>,
        bank_info: &'accounts AccountInfo<'accounts>,
        recipient: &AccountInfo<'accounts>,
        write_off_acc_info: &AccountInfo<'accounts>,
        token_program: &'accounts AccountInfo<'accounts>,
        amount: u64,
    ) -> ProgramResult {
        let token_market = TokenMarket::try_from_slice(*market_info.data.borrow())?;
        if !token_market.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }

        let write_off_acc = Account::unpack_from_slice(*write_off_acc_info.data.borrow())?;
        if !write_off_acc.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }
        if write_off_acc.mint != token_market.mint_of_acceptable {
            return Err(ProgramError::InvalidAccountData);
        }

        let recipient_acc = Account::unpack_from_slice(*recipient.data.borrow())?;
        if !recipient_acc.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }
        if recipient_acc.mint != token_market.emitter_mint {
            return Err(ProgramError::InvalidAccountData);
        }

        // check that there are enough tokens to exchange the requested number of tokens
        if write_off_acc.amount < amount {
            return Err(ProgramError::InsufficientFunds);
        }
        
        invoke(
            &transfer(
                &token_program.key,
                write_off_acc_info.key,
                &token_market.bank,
                &token_market.authority,
                &[&token_market.authority],
                amount,
            )?,
            &[
                token_program.clone(),
                write_off_acc_info.clone(),
                bank_info.clone(),
            ],
        )?;

        invoke(
            &mint_to(
                &token_program.key,
                &token_market.emitter_mint,
                &recipient.key,
                &token_market.authority,
                &[&token_market.authority],
                amount,
            )?,
            &[
                token_program.clone(),
                market_info.clone(),
                recipient.clone(),
            ],
        )?;

        Ok(())
    }
}
