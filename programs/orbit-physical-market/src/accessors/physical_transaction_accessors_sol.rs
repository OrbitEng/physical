use anchor_lang::solana_program::{
    system_instruction::transfer,
    program::invoke
};
use anchor_lang::{
    prelude::*,
    AccountsClose
};
use market_accounts::structs::market_account::OrbitMarketAccount;
use transaction::{
    transaction_struct::TransactionState,
    transaction_trait::OrbitTransactionTrait
};
use crate::structs::{
    physical_transaction::PhysicalTransaction,
    physical_product::PhysicalProduct
};
use crate::errors::PhysicalMarketErrors;
use crate::accessors::common::close_escrow;
use dispute::{
    structs::{
        dispute_trait::OrbitDisputableTrait,
        dispute_struct::{
            OrbitDispute,
            DisputeState
        }
    },
    program::Dispute,
    cpi::accounts::{
        OpenDispute,
        CloseDispute
    }
};

////////////////////////////////////////////////////////////////////
/// ORBIT BASE TRANSACTION FUNCTIONALITIES
#[derive(Accounts)]
pub struct OpenPhysicalTransaction<'info>{
    #[account(
        init,
        payer = buyer_wallet,
        space = 1000
    )]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    pub phys_product: Account<'info, PhysicalProduct>,

    #[account(
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: SystemAccount<'info>,

    #[account(
        constraint = buyer_account.wallet == buyer_wallet.key()
    )]
    pub buyer_account: Account<'info, OrbitMarketAccount>,

    #[account(mut)]
    pub buyer_wallet: Signer<'info>,

    pub master_pubkey: Signer<'info>,

    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct ClosePhysicalTransaction<'info>{
    #[account(
        mut,
        constraint = phys_transaction.metadata.escrow_account == escrow_account.key(),
        constraint = phys_transaction.metadata.seller == seller_account.key(),
        constraint = phys_transaction.metadata.buyer == buyer_account.key(),
        constraint = phys_transaction.metadata.transaction_state == TransactionState::BuyerConfirmedProduct,
    )]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    pub buyer_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>,

    pub seller_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut,
        address = seller_account.wallet
    )]
    pub seller_wallet: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: SystemAccount<'info>,

    #[account(
        constraint = (authority.key() == seller_account.master_pubkey) || (authority.key() == buyer_account.master_pubkey)
    )] // enforce constraints. only certain people should be able to close
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct FundEscrow<'info>{
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::SellerConfirmed,
        constraint = phys_transaction.metadata.escrow_account == escrow_account.key()
    )]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        address = phys_transaction.metadata.buyer
    )]
    pub buyer_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: SystemAccount<'info>,

    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: Signer<'info>
}

impl<'a, 'b, 'c> OrbitTransactionTrait<'a, 'b, 'c, OpenPhysicalTransaction<'a>, ClosePhysicalTransaction<'b>, FundEscrow<'c>> for PhysicalTransaction{
    fn open(ctx: Context<OpenPhysicalTransaction>, price: u64) -> Result<()>{
        ctx.accounts.phys_transaction.metadata.buyer = ctx.accounts.buyer_account.key();
        ctx.accounts.phys_transaction.metadata.seller = ctx.accounts.phys_product.metadata.seller.key();
        ctx.accounts.phys_transaction.metadata.product = ctx.accounts.phys_product.key();
        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Opened;
        ctx.accounts.phys_transaction.metadata.transaction_price = price;

        ctx.accounts.phys_transaction.metadata.funded = false;

        ctx.accounts.phys_transaction.metadata.escrow_account = ctx.accounts.escrow_account.key();
        Ok(())
    }

    fn close(ctx: Context<ClosePhysicalTransaction>) -> Result<()>{
        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Closed;
        
        let res = match ctx.bumps.get("escrow_account"){
            Some(escrow_seeds) => close_escrow(
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.seller_wallet.to_account_info(),
                &[&[b"orbit_escrow_account", ctx.accounts.phys_transaction.key().as_ref(), &[*escrow_seeds]]],
                100 // to change to 95%. 5% to orbit :D
            ),
            None => return err!(PhysicalMarketErrors::InvalidEscrowBump)
        };
        if res.is_err(){return res};
        ctx.accounts.phys_transaction.close(ctx.accounts.buyer_wallet.to_account_info()).map_err(|e| e)
    }

    fn fund_escrow(ctx: Context<FundEscrow>) -> Result<()>{
        invoke(
            &transfer(
                &ctx.accounts.buyer_wallet.key(),
                &ctx.accounts.escrow_account.key(),
                ctx.accounts.phys_transaction.metadata.transaction_price
            ),
            &[
                ctx.accounts.buyer_wallet.to_account_info(),
                ctx.accounts.escrow_account.to_account_info()
            ]
        ).expect("could not fund escrow");
        ctx.accounts.phys_transaction.metadata.funded = true;
        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::BuyerFunded;
        Ok(())
    }
}


////////////////////////////////////////////////////////////////////
/// ORBIT DISPUTE FUNCTIONALITIES
#[derive(Accounts)]
pub struct OpenPhysicalDispute<'info>{
    #[account(
        seeds = [
            b"dispute_account",
            phys_transaction.key().as_ref()
        ],
        seeds::program = dispute::ID,
        bump
    )]
    pub new_dispute: SystemAccount<'info>,

    #[account(
        mut,
        constraint =
        (phys_transaction.metadata.transaction_state == TransactionState::BuyerFunded) ||
        (phys_transaction.metadata.transaction_state == TransactionState::Shipped) ||
        (phys_transaction.metadata.transaction_state == TransactionState::BuyerConfirmedDelivery)
    )]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        // opener must be buyer or seller
        constraint = (opener.key() == phys_transaction.metadata.seller) || (opener.key() == phys_transaction.metadata.buyer),
        // payer must be related to opener
        constraint = payer.key() == opener.wallet
    )]
    //
    pub opener: Account<'info, OrbitMarketAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub physical_auth: SystemAccount<'info>,

    pub dispute_program: Program<'info, Dispute>,

    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct ClosePhysicalDispute<'info>{
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::Frozen,
        constraint = (phys_transaction.metadata.seller == favor_market_account.key()) || (phys_transaction.metadata.buyer == favor_market_account.key()),
        constraint = phys_transaction.metadata.escrow_account == escrow_account.key()
    )]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        mut,
        constraint = phys_dispute.dispute_transaction == phys_transaction.key(),
        constraint = phys_dispute.dispute_state == DisputeState::Resolved,
        has_one = favor,
        has_one = funder
    )]
    pub phys_dispute: Account<'info, OrbitDispute>,

    #[account(
        mut,
        // can not be default side
        constraint = favor.key() != Pubkey::new_from_array([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])
    )]
    pub favor: SystemAccount<'info>,

    #[account(
        constraint = favor_market_account.wallet == favor.key()
    )]
    pub favor_market_account: Account<'info, OrbitMarketAccount>,

    #[account(mut)]
    pub funder: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: SystemAccount<'info>,

    pub physical_auth: SystemAccount<'info>,

    pub dispute_program: Program<'info, Dispute>,
}

impl<'a, 'b> OrbitDisputableTrait<'a, 'b, OpenPhysicalDispute<'a>, ClosePhysicalDispute<'b>> for PhysicalTransaction{
    fn open_dispute(ctx: Context<OpenPhysicalDispute>, threshold: u8) -> Result<()>{
        if (!ctx.accounts.new_dispute.data_is_empty()) || (ctx.accounts.new_dispute.lamports() > 0){
            return err!(PhysicalMarketErrors::DisputeExists)
        }

        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Frozen;

        let res: Result<()> = match ctx.bumps.get("physical_auth"){
            Some(signer_bump) => dispute::cpi::open_dispute(
                CpiContext::new_with_signer(
                    ctx.accounts.dispute_program.to_account_info(),
                    OpenDispute{
                        new_dispute: ctx.accounts.new_dispute.to_account_info(),
                        in_transaction: ctx.accounts.phys_transaction.to_account_info(),
                        caller: ctx.accounts.physical_auth.to_account_info(),
                        payer: ctx.accounts.payer.to_account_info(),
                        system_program: ctx.accounts.system_program.to_account_info()
                    },
                    &[&[b"physical_auth", &[*signer_bump]]]
                ),
                threshold as usize
            ),
            None => return err!(PhysicalMarketErrors::InvalidAuthBump)
        };

        if res.is_err(){return res};
        Ok(())
    }
    
    fn close_dispute(ctx: Context<ClosePhysicalDispute>) -> Result<()>{
        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Closed;

        let mut res: Result<()> = match ctx.bumps.get("physical_auth"){
            Some(signer_bump) => 
                dispute::cpi::close_dispute(
                    CpiContext::new_with_signer(
                        ctx.accounts.dispute_program.to_account_info(),
                        CloseDispute{
                            dispute_account: ctx.accounts.phys_dispute.to_account_info(),
                            funder: ctx.accounts.funder.to_account_info(),
                            caller: ctx.accounts.physical_auth.to_account_info()
                        },
                        &[&[b"physical_auth", &[*signer_bump]]]
                    )
                ).map_err(|e| anchor_lang::error::Error::from(e))
            ,
            None => return err!(PhysicalMarketErrors::InvalidAuthBump)
        };
        if res.is_err(){return res};


        res = match ctx.bumps.get("escrow_account"){
            Some(escrow_bump) => 
            close_escrow(
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.favor.to_account_info(),
                &[&[b"orbit_escrow_account", ctx.accounts.phys_transaction.key().as_ref(), &[*escrow_bump]]],
                100
            ),
            None => return err!(PhysicalMarketErrors::InvalidEscrowBump)
        };
        if res.is_err(){return res};

        ctx.accounts.phys_transaction.close(ctx.accounts.favor.to_account_info())
    }
}

///////////////////////////////////////////////////////////////////////////////////////
/// SELLER CONFIRMATIONS

#[derive(Accounts)]
pub struct UpdateShipping<'info>{
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::BuyerFunded
    )]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        address = phys_transaction.metadata.seller
    )]
    pub seller_account: Account<'info, OrbitMarketAccount>,

    #[account(
        address = seller_account.master_pubkey
    )]
    pub signer_auth: Signer<'info>,
}

pub fn update_shipping(ctx: Context<UpdateShipping>, enc_shipping: [u8; 64]) -> Result<()>{
    ctx.accounts.phys_transaction.shipping = enc_shipping;
    ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Shipped;
    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////////////////
/// BUYER CONFIRMATIONS

#[derive(Accounts)]
pub struct BuyerConfirm<'info>{
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::Shipped
    )]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        address = phys_transaction.metadata.buyer
    )]
    pub buyer_account: Account<'info, OrbitMarketAccount>,

    #[account(
        address = buyer_account.master_pubkey
    )]
    pub signer_auth: Signer<'info>,
}

pub fn confirm_delivery(ctx: Context<BuyerConfirm>) -> Result<()>{
    ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::BuyerConfirmedDelivery;
    Ok(())
}

pub fn confirm_product(ctx: Context<BuyerConfirm>) -> Result<()>{
    ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::BuyerConfirmedProduct;
    Ok(())
}