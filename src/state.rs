use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Escrow {
    pub is_initialized: bool,
    pub authority: Pubkey,
    pub sell_mint: Pubkey,
    pub buy_mint: Pubkey,
    pub sell_amount: u64,
    pub buy_amount: u64,
    pub receive_account: Pubkey,
    pub bump: u8,
}

impl Escrow {
    pub fn new(
        authority: Pubkey,
        sell_mint: Pubkey,
        buy_mint: Pubkey,
        sell_amount: u64,
        buy_amount: u64,
        receive_account: Pubkey,
        bump: u8,
    ) -> Self {
        Self {
            is_initialized: true,
            authority,
            sell_mint,
            buy_mint,
            sell_amount,
            buy_amount,
            receive_account,
            bump,
        }
    }
    /// The length of the Escrow struct in bytes
    pub fn len() -> usize {
        // is_initialized: 1 byte (bool)
        1_ + // authority: 32 bytes (Pubkey)
        32 + // sell_mint: 32 bytes (Pubkey)
        32 + // buy_mint: 32 bytes (Pubkey)
        32 + // sell_amount: 8 bytes (u64)
        8 + // buy_amount: 8 bytes (u64)
        8 + // receive_account: 32 bytes (Pubkey)
        32 + // bump: 1 byte (u8)
        1
    }
}
