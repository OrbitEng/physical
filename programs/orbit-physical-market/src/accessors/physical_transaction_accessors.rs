use anchor_lang::{
    prelude::*,
    AccountsClose,
    solana_program::{
        system_instruction::transfer,
        program::invoke_signed
    }
};

use market_accounts::structs::market_account::OrbitMarketAccount;
use crate::structs::physical_transaction::PhysicalTransaction;
use transaction::transaction_struct::TransactionState;

// todo:
//      add default escrow


////////////////////////////////////////////////////////////////////
/// ORBIT BASE TRANSACTION FUNCTIONALITIES
#[derive(Accounts)]
pub struct FreezePhysicalTransaction<'info>{
    #[account(mut)]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    // todo:
    //      use dispute crate. enforce that this
    //      authority.key() == seeds["some_seed_string", dispute::ID(), &[bumps]]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UnfreezePhysicalTransaction<'info>{
    #[account(mut)]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    // todo:
    //      use dispute crate. enforce that this
    //      authority.key() == seeds["some_seed_string", dispute::ID(), &[bumps]]
    pub authority: Signer<'info>,   
}

#[derive(Accounts)]
pub struct OpenPhysicalTransaction<'info>{
    #[account(
        init,
        payer = payer,
        space = 1000
    )]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        init,
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref()
        ],
        payer = payer,
        space = 32,
        bump
    )]
    pub escrow_account: AccountInfo<'info>,

    // so that other people cant open transactions on this buyer's behalf
    // that's a one stop shop into framing-ville
    #[account(
        constraint = buyer.wallet == payer.key()
    )]
    pub buyer: Account<'info, OrbitMarketAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub master_pubkey: Signer<'info>,

    pub system_program: Program<'info, System>
}


#[derive(Accounts)]
pub struct ClosePhysicalTransaction<'info>{
    #[account(mut)]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(mut)]
    pub destination: AccountInfo<'info>,

    #[account(mut)]
    pub escrow_account: AccountInfo<'info>,

    #[account()] // enforce constraints. only certain people should be able to close
    pub authority: Signer<'info>,
}

pub fn close_physical_transaction_handler(ctx: Context<ClosePhysicalTransaction>) -> Result<()>{
    // fix
    // let close_escrow = invoke_signed(
    //     &transfer(
    //         ctx.accounts.escrow_account.key,
    //         ctx.accounts.destination.key,
    //         ctx.accounts.escrow_account.lamports()
    //     ), &[
    //         ctx.accounts.escrow_account.clone(),
    //         ctx.accounts.destination.clone()
    //     ], 
    //     &[
    //         &[b"orbit_escrow_account",
    //         ctx.accounts.phys_transaction.key().as_ref(),
    //         &[
    //             *ctx.bumps.get("jerone").unwrap()
    //         ]]
    //     ]
    // );
    // match close_escrow{
    //     Ok(_) => {},
    //     Err(e) => return Err(e)
    // };
    ctx.accounts.phys_transaction.close(ctx.accounts.destination.clone()).map_err(|e| e)
}