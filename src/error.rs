use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum EscrowError {
    #[error("This account should be Signer")]
    InvalidSigner,
    #[error("Wrong Program Provided")]
    InvalidProgram,
    #[error("Wrong Mint Provided")]
    InvalidMint,
    #[error("Wrong Token Account Provided")]
    InvalidTokenAccount,
    #[error("PDA is wrong")]
    InvalidPda,
    #[error("Overflow occured in addition")]
    AdditionOverflow,
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
