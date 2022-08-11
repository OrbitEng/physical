use anchor_lang::{
    prelude::*,
    AccountsClose
};
use market_accounts::structs::market_account::OrbitMarketAccount;
use transaction::{transaction_struct::TransactionState, transaction_trait::OrbitTransactionTrait};
use crate::structs::{physical_transaction::PhysicalTransaction, physical_product::PhysicalProduct};
use dispute::structs::dispute_trait::OrbitDisputableTrait;

////////////////////////////////////////////////////////////////////
/// ORBIT BASE TRANSACTION FUNCTIONALITIES
#[derive(Accounts)]
pub struct OpenPhysicalTransaction<'info>{
    #[account(
        init,
        payer = payer,
        space = 1000
    )]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    pub phys_product: Account<'info, PhysicalProduct>,

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
    #[account(
        mut,
        has_one = escrow_account
    )]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        mut,
        address = phys_transaction.metadata.seller
    )]
    pub destination: AccountInfo<'info>,

    #[account(mut)]
    pub escrow_account: AccountInfo<'info>,

    #[account(
        constraint = (authority.key() == phys_transaction.metadata.buyer) || (authority.key() == phys_transaction.metadata.seller)
    )] // enforce constraints. only certain people should be able to close
    pub authority: Signer<'info>,
}

impl<'a, 'b> OrbitTransactionTrait<'a, 'b, OpenPhysicalTransaction<'a>, ClosePhysicalTransaction<'b>> for PhysicalTransaction{
    fn open(ctx: Context<OpenPhysicalTransaction>, price: u64) -> Result<()>{
        ctx.accounts.phys_transaction.metadata.buyer = ctx.accounts.buyer.key();
        ctx.accounts.phys_transaction.metadata.seller = ctx.accounts.phys_product.metadata.seller.key();
        ctx.accounts.phys_transaction.metadata.product = ctx.accounts.phys_product.key();
        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Opened;
        ctx.accounts.phys_transaction.metadata.transaction_price = price;

        ctx.accounts.phys_transaction.escrow_account = ctx.accounts.escrow_account.key();
        Ok(())
    }

    fn close(ctx: Context<ClosePhysicalTransaction>) -> Result<()>{
        // let escrow_amt = ctx.accounts.escrow_account.lamports();
    
        // todo: add fees
        let xfer_amt = ctx.accounts.phys_transaction.metadata.transaction_price;
        **ctx.accounts.escrow_account.try_borrow_mut_lamports()? -= xfer_amt;
        **ctx.accounts.destination.try_borrow_mut_lamports()? += xfer_amt;
        ctx.accounts.phys_transaction.close(ctx.accounts.destination.clone()).map_err(|e| e)
    }
}


////////////////////////////////////////////////////////////////////
/// ORBIT DISPUTE FUNCTIONALITIES
#[derive(Accounts)]
pub struct OpenDispute<'info>{
    #[account(mut)]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        seeds = [
            b"dispute_auth"
        ],
        bump,
        seeds::program = dispute::ID,
    )]
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct CloseDispute<'info>{
    #[account(mut)]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(mut)]
    pub favor: AccountInfo<'info>,

    #[account(
        seeds = [
            b"dispute_auth"
        ],
        bump,
        seeds::program = dispute::ID,
    )]
    pub authority: Signer<'info>,   
}
// do checks
impl<'a, 'b> OrbitDisputableTrait<'a, 'b, OpenDispute<'a>, CloseDispute<'b>>{
    fn open_dispute(ctx: Context<OpenDispute>) -> Result<()>{
        ctx.accounts.phys_transaction.metadata.transaction_state == TransactionState::Frozen;
        Ok(())
    }

    
    fn close_dispute(ctx: Context<CloseDispute>) -> Result<()>{
        ctx.accounts.phys_transaction.metadata.transaction_state == TransactionState::Closed;
        ctx.accounts.phys_transaction.close(ctx.accounts.favor);
        Ok(())
    }
}