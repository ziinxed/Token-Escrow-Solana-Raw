use crate::{error::EscrowError, state::Escrow};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::{
    instruction::{close_account, transfer_checked},
    state::Mint,
};

pub fn process_exchange(
    accounts: &[AccountInfo],
    sell_amount: u64,
    buy_amount: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    msg!("Instruction : Exchange");

    let account_info_iter = &mut accounts.iter();

    let authority = next_account_info(account_info_iter)?;

    let taker = next_account_info(account_info_iter)?;

    let taker_sell_mint = next_account_info(account_info_iter)?;

    let sell_mint_info = Mint::unpack(&taker_sell_mint.data.borrow())?;

    let taker_buy_mint = next_account_info(account_info_iter)?;

    let buy_mint_info = Mint::unpack(&taker_buy_mint.data.borrow())?;

    let taker_sell_mint_ata = next_account_info(account_info_iter)?;

    let taker_buy_mint_ata = next_account_info(account_info_iter)?;

    let receiver_account = next_account_info(account_info_iter)?;

    let escrow_account = next_account_info(account_info_iter)?;

    let escrow_token_account = next_account_info(account_info_iter)?;

    let token_program = next_account_info(account_info_iter)?;

    // 1. transfer sell amount of taker sell mint from taker to receiver account

    let tranfer_from_taker_to_authority_ix = transfer_checked(
        token_program.key,
        taker_sell_mint_ata.key,
        taker_sell_mint.key,
        receiver_account.key,
        taker.key,
        &[taker.key],
        sell_amount,
        sell_mint_info.decimals,
    )
    .unwrap();

    invoke(
        &tranfer_from_taker_to_authority_ix,
        &[
            taker_sell_mint_ata.clone(),
            taker_sell_mint.clone(),
            receiver_account.clone(),
            taker.clone(),
            token_program.clone(),
        ],
    )?;

    // 2. transfer by amount of taker buy mint from escrow token account to taker

    let escrow_info = Escrow::try_from_slice(&escrow_account.data.borrow())?;

    let escrow_signer_seed: &[&[&[u8]]] = &[&[
        b"escrow",
        authority.key.as_ref(),
        taker_buy_mint.key.as_ref(),
        &[escrow_info.bump],
    ]];

    let tranfer_from_escrow_token_account_to_taker_ix = transfer_checked(
        token_program.key,
        escrow_token_account.key,
        taker_buy_mint.key,
        taker_buy_mint_ata.key,
        escrow_account.key,
        &[escrow_account.key],
        buy_amount,
        buy_mint_info.decimals,
    )
    .unwrap();

    invoke_signed(
        &tranfer_from_escrow_token_account_to_taker_ix,
        &[
            escrow_token_account.clone(),
            taker_buy_mint.clone(),
            taker_buy_mint_ata.clone(),
            escrow_account.clone(),
            token_program.clone(),
        ],
        escrow_signer_seed,
    )?;
    // 3. close escrow token account

    let close_escrow_token_account_ix = close_account(
        token_program.key,
        escrow_token_account.key,
        authority.key,
        escrow_account.key,
        &[escrow_account.key],
    )
    .unwrap();

    invoke_signed(
        &close_escrow_token_account_ix,
        &[
            escrow_token_account.clone(),
            authority.clone(),
            escrow_account.clone(),
        ],
        escrow_signer_seed,
    )?;

    // 4. close escrow account

    **authority.try_borrow_mut_lamports()? = authority
        .lamports()
        .checked_add(escrow_account.lamports())
        .ok_or(EscrowError::AdditionOverflow)?;

    **escrow_account.try_borrow_mut_lamports()? = 0;
    *escrow_account.try_borrow_mut_data()? = &mut [];

    Ok(())
}
