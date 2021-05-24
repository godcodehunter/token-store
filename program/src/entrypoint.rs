//! Program entrypoint

#![cfg(all(target_arch = "bpf", not(feature = "no-entrypoint")))]

use crate::{error::TokenMarketError, processor::Processor};
use solana_program::program_error::PrintProgramError;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

entrypoint!(process_instruction);
fn process_instruction<'accounts>(
    program_id: &Pubkey,
    accounts: &'accounts [AccountInfo<'accounts>],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = Processor::process_instruction(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<TokenMarketError>();
        return Err(error);
    }
    Ok(())
}
