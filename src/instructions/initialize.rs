use crate::{error::EscrowError, state::Escrow};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction,
    system_program::ID as SYSTEM_PROGRAM_ID,
    sysvar::{rent::Rent, Sysvar},
};
use spl_associated_token_account::{
    instruction::create_associated_token_account, ID as ASSOCIATED_TOKEN_PROGRAM_ID,
};
use spl_token::{
    instruction::transfer_checked, state::Account as TokenAccount, state::Mint,
    ID as TOKEN_PROGRAM_ID,
};

pub fn process_init_escrow(
    accounts: &[AccountInfo],
    sell_amount: u64,
    buy_amount: u64,
    program_id: &Pubkey,
) -> ProgramResult {
    msg!("Instruction : InitEscrow");

    let account_info_iter = &mut accounts.iter();

    let authority = next_account_info(account_info_iter)?;

    if !authority.is_signer {
        return Err(EscrowError::InvalidSigner.into());
    }

    let sell_mint = next_account_info(account_info_iter)?;

    let sell_mint_info = Mint::unpack(&sell_mint.data.borrow())?;

    if !sell_mint_info.is_initialized || sell_mint.owner != &TOKEN_PROGRAM_ID {
        return Err(EscrowError::InvalidMint.into());
    }

    let buy_mint = next_account_info(account_info_iter)?;

    let buy_mint_info = Mint::unpack(&buy_mint.data.borrow())?;

    if !buy_mint_info.is_initialized || buy_mint.owner != &TOKEN_PROGRAM_ID {
        return Err(EscrowError::InvalidMint.into());
    }

    let authority_sell_mint_ata = next_account_info(account_info_iter)?;

    let authority_sell_mint_ata_info =
        TokenAccount::unpack(&authority_sell_mint_ata.data.borrow())?;

    if authority_sell_mint_ata_info.is_frozen()
        || authority_sell_mint_ata.owner != &TOKEN_PROGRAM_ID
    {
        return Err(EscrowError::InvalidTokenAccount.into());
    }

    let authority_buy_mint_ata = next_account_info(account_info_iter)?;

    let authority_buy_mint_ata_info = TokenAccount::unpack(&authority_buy_mint_ata.data.borrow())?;

    if authority_buy_mint_ata_info.is_frozen() || authority_buy_mint_ata.owner != &TOKEN_PROGRAM_ID
    {
        return Err(EscrowError::InvalidTokenAccount.into());
    }

    let escrow_account = next_account_info(account_info_iter)?;

    let escrow_token_account = next_account_info(account_info_iter)?;

    let rent_account = next_account_info(account_info_iter)?;

    let system_program = next_account_info(account_info_iter)?;

    if system_program.key != &SYSTEM_PROGRAM_ID {
        return Err(EscrowError::InvalidProgram.into());
    }

    let token_program = next_account_info(account_info_iter)?;

    if token_program.key != &TOKEN_PROGRAM_ID {
        return Err(EscrowError::InvalidProgram.into());
    }

    let associated_token_program = next_account_info(account_info_iter)?;

    if associated_token_program.key != &ASSOCIATED_TOKEN_PROGRAM_ID {
        return Err(EscrowError::InvalidProgram.into());
    }

    let rent = &Rent::from_account_info(rent_account)?;

    let required_lamports = rent.minimum_balance(Escrow::len());

    let (escrow_key, bump) = Pubkey::find_program_address(
        &[b"escrow", authority.key.as_ref(), sell_mint.key.as_ref()],
        program_id,
    );

    if escrow_account.key != &escrow_key {
        return Err(EscrowError::InvalidPda.into());
    }

    let escrow_account_create_ix = system_instruction::create_account(
        authority.key,
        escrow_account.key,
        required_lamports,
        Escrow::len() as u64,
        program_id,
    );

    let escrow_signer_seed: &[&[&[u8]]] = &[&[
        b"escrow",
        authority.key.as_ref(),
        sell_mint.key.as_ref(),
        &[bump],
    ]];

    invoke_signed(
        &escrow_account_create_ix,
        &[
            authority.clone(),
            escrow_account.clone(),
            system_program.clone(),
        ],
        escrow_signer_seed,
    )?;

    let escrow_data = Escrow::new(
        *authority.key,
        *sell_mint.key,
        *buy_mint.key,
        sell_amount,
        buy_amount,
        *authority_buy_mint_ata.key,
        bump,
    );

    escrow_data.serialize(&mut &mut escrow_account.data.borrow_mut()[..])?;

    // 2. create escrow ata for sell mint

    let escrow_ata_create_ix = create_associated_token_account(
        authority.key,
        escrow_account.key,
        sell_mint.key,
        &TOKEN_PROGRAM_ID,
    );

    invoke(
        &escrow_ata_create_ix,
        &[
            authority.clone(),
            escrow_token_account.clone(),
            escrow_account.clone(),
            sell_mint.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
    )?;

    let tranfer_amount_ix = transfer_checked(
        &TOKEN_PROGRAM_ID,
        authority_sell_mint_ata.key,
        sell_mint.key,
        escrow_token_account.key,
        authority.key,
        &[authority.key],
        sell_amount,
        sell_mint_info.decimals,
    )
    .unwrap();

    invoke(
        &tranfer_amount_ix,
        &[
            authority_sell_mint_ata.clone(),
            sell_mint.clone(),
            escrow_token_account.clone(),
            authority.clone(),
            token_program.clone(),
        ],
    )?;

    Ok(())
}
