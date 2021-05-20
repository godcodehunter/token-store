//! Error types

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use solana_program::{
    decode_error::DecodeError,
    msg,
    program_error::{PrintProgramError, ProgramError},
};
use thiserror::Error;

/// Errors that may be returned by the `Token market` program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum TokenMarketError {
    #[error("insufficient funds")]
    InsufficientFundsError,
}
impl From<TokenMarketError> for ProgramError {
    fn from(e: TokenMarketError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for TokenMarketError {
    fn type_of() -> &'static str {
        "TokenMarketError"
    }
}

impl PrintProgramError for TokenMarketError {
    fn print<E>(&self)
    where
        E: 'static + std::error::Error + DecodeError<E> + PrintProgramError + FromPrimitive,
    {
        match self {
            TokenMarketError::InsufficientFundsError => msg!("Example error message"),
        }
    }
}
