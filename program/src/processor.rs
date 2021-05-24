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
    instruction::{mint_to, set_authority, transfer},
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
                let token_market_info = next_account_info(account_info_iter)?;
                let bank_info = next_account_info(account_info_iter)?;
                let emitter_info = next_account_info(account_info_iter)?;
                let accepted_mint_info = next_account_info(account_info_iter)?;
                Self::process_init_market(
                    program_id,
                    owner_info,
                    token_market_info,
                    bank_info,
                    emitter_info,
                    accepted_mint_info,
                )
            }
            TokenMarketInstructions::BuyTokens { amount } => {
                msg!("Instruction: BuyTokens");

                let token_market_info = next_account_info(account_info_iter)?;
                let bank_info = next_account_info(account_info_iter)?;
                let holder_info = next_account_info(account_info_iter)?;
                let holder_owner = next_account_info(account_info_iter)?;
                let recipient = next_account_info(account_info_iter)?;
                let token_program = next_account_info(account_info_iter)?;
                Self::process_buy_tokens(
                    program_id,
                    token_market_info,
                    bank_info,
                    holder_info,
                    holder_owner,
                    recipient,
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
        mut token_market_info: &AccountInfo,
        bank_info: &AccountInfo,
        emitter_info: &AccountInfo,
        accepted_mint_info: &AccountInfo,
    ) -> ProgramResult {
        let token_market = TokenMarket::try_from_slice(&token_market_info.data.borrow())?;
        if token_market.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        TokenMarket {
            is_initialized: true,
            token_bank: *bank_info.key,
            tradable_token_mint: *emitter_info.key,
            authority: Pubkey::find_program_address(&[b"token-market"], program_id).0,
            accept_token_mint: *accepted_mint_info.key,
        }
        .serialize(&mut *token_market_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_buy_tokens<'accounts>(
        program_id: &Pubkey,
        token_market_info: &'accounts AccountInfo<'accounts>,
        bank_info: &AccountInfo<'accounts>,
        holder_info: &AccountInfo<'accounts>,
        holder_owner: &AccountInfo<'accounts>,
        recipient: &AccountInfo<'accounts>,
        token_program: &'accounts AccountInfo<'accounts>,
        amount: u64,
    ) -> ProgramResult {
        let token_market = TokenMarket::try_from_slice(*token_market_info.data.borrow())?;
        if !token_market.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }

        let holder_acc = Account::unpack_from_slice(*holder_info.data.borrow())?;
        if !holder_acc.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }

        if holder_acc.mint != token_market.accept_token_mint {
            return Err(ProgramError::InvalidAccountData);
        }

        let recipient_acc = Account::unpack_from_slice(*recipient.data.borrow())?;
        if !recipient_acc.is_initialized() {
            return Err(ProgramError::UninitializedAccount);
        }
        if recipient_acc.mint != token_market.tradable_token_mint {
            return Err(ProgramError::InvalidAccountData);
        }

        // check that there are enough tokens to exchange the requested number of tokens
        if holder_acc.amount < amount {
            return Err(ProgramError::InsufficientFunds);
        }

        invoke(
            &set_authority(
                &token_program.key,
                holder_info.key,
                Some(&token_market.authority),
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

        Self::swap_tokens(
            token_program,
            token_market,
            token_market_info,
            holder_owner,
            holder_info,
            bank_info,
            recipient,
            amount,
        )?;

        Ok(())
    }

    fn swap_tokens<'accounts>(
        token_program: &'accounts AccountInfo<'accounts>,
        token_market: TokenMarket,
        token_market_info: &'accounts AccountInfo<'accounts>,
        mut holder_owner: &AccountInfo<'accounts>,
        mut holder_info: &AccountInfo<'accounts>,
        bank_info: &AccountInfo<'accounts>,
        recipient: &AccountInfo<'accounts>,
        amount: u64,
    ) -> Result<(), ProgramError> {
        invoke(
            &transfer(
                &token_program.key,
                holder_info.key,
                &token_market.token_bank,
                &token_market.authority,
                &[&token_market.authority],
                amount,
            )?,
            &[
                token_program.clone(),
                holder_info.clone(),
                bank_info.clone(),
            ],
        )?;

        **holder_owner.lamports.borrow_mut() += holder_info.lamports();
        **holder_info.lamports.borrow_mut() = 0;

        invoke(
            &mint_to(
                &token_program.key,
                &token_market.tradable_token_mint,
                &recipient.key,
                &token_market.tradable_token_mint,
                &[&token_market.authority],
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
}
