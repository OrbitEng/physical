use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    system_instruction::transfer,
    program::invoke_signed
};

pub fn close_escrow<'a>(escrow_account: AccountInfo<'a>, destination: AccountInfo<'a>, seeds: &[&[&[u8]]], rate: u8) -> Result<()>{
    invoke_signed(
        &transfer(
            &escrow_account.key(),
            &destination.key(),
            (escrow_account.lamports() * rate as u64) / 100
        ),
        &[
            escrow_account,
            destination
        ],
        seeds
    ).map_err(|e| anchor_lang::error::Error::from(e))
    
}