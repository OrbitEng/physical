use anchor_lang::{
    prelude::*,
    AccountsClose,
    solana_program::{
        system_instruction::transfer,
        program::invoke
    },
};
use market_accounts::{
    structs::{
        market_account::OrbitMarketAccount,
        OrbitMarketAccountTrait,
        ReviewErrors
    },
    program::OrbitMarketAccounts,
    MarketAccountErrors
};
use transaction::{
    transaction_struct::TransactionState,
    transaction_trait::OrbitTransactionTrait
};
use crate::{
    PhysicalTransaction,
    errors::PhysicalMarketErrors,
    close_escrow_sol,
    close_escrow_spl,
    
    OpenPhysicalTransactionSol,
    ClosePhysicalTransactionSol,
    FundEscrowSol,
    ClosePhysicalDisputeSol,

    OpenPhysicalTransactionSpl,
    ClosePhysicalTransactionSpl,
    FundEscrowSpl,
    ClosePhysicalDisputeSpl,

    post_tx_incrementing,
    close_dispute_helper,
    submit_rating_with_signer,

    id, program::OrbitPhysicalMarket
};
use dispute::{
    structs::{
        dispute_trait::OrbitDisputableTrait
    },
    program::Dispute,
    cpi::accounts::OpenDispute
};

////////////////////////////////////////////////////////////////////
/// ORBIT BASE TRANSACTION FUNCTIONALITIES

#[derive(Accounts)]
pub struct CloseTransactionAccount<'info>{
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::Closed,
    )]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        constraint = 
            (market_account.key() == phys_transaction.metadata.buyer) ||
            (market_account.key() == phys_transaction.metadata.seller)
    )]
    pub market_account: Account<'info, OrbitMarketAccount>,

    #[account(
        address = market_account.master_pubkey
    )]
    pub account_auth: Signer<'info>,

    #[account(
        mut,
        address = market_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>
}

impl<'a, 'b, 'c, 'd, 'e, 'f, 'g> OrbitTransactionTrait<'a, 'b, 'c, 'd, 'e, 'f, 'g, OpenPhysicalTransactionSol<'a>, OpenPhysicalTransactionSpl<'b>, ClosePhysicalTransactionSol<'c>, ClosePhysicalTransactionSpl<'d>, FundEscrowSol<'e>, FundEscrowSpl<'f>, CloseTransactionAccount<'g>> for PhysicalTransaction{
    fn open_sol(ctx: Context<OpenPhysicalTransactionSol>, price: u64) -> Result<()>{
        ctx.accounts.phys_transaction.metadata.buyer = ctx.accounts.buyer_account.key();
        ctx.accounts.phys_transaction.metadata.seller = ctx.accounts.phys_product.metadata.seller.key();
        ctx.accounts.phys_transaction.metadata.product = ctx.accounts.phys_product.key();
        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Opened;
        ctx.accounts.phys_transaction.metadata.transaction_price = price;
        ctx.accounts.phys_transaction.metadata.currency = ctx.accounts.phys_product.metadata.currency;

        ctx.accounts.phys_transaction.metadata.funded = false;

        ctx.accounts.phys_transaction.metadata.escrow_account = ctx.accounts.escrow_account.key();
        Ok(())
    }

    fn open_spl(ctx: Context<OpenPhysicalTransactionSpl>, price: u64) -> Result<()>{
        ctx.accounts.phys_transaction.metadata.buyer = ctx.accounts.buyer_account.key();
        ctx.accounts.phys_transaction.metadata.seller = ctx.accounts.phys_product.metadata.seller.key();
        ctx.accounts.phys_transaction.metadata.product = ctx.accounts.phys_product.key();
        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Opened;
        ctx.accounts.phys_transaction.metadata.transaction_price = price;
        ctx.accounts.phys_transaction.metadata.currency = ctx.accounts.phys_product.metadata.currency;

        ctx.accounts.phys_transaction.metadata.funded = false;

        ctx.accounts.phys_transaction.metadata.escrow_account = ctx.accounts.escrow_account.key();
        Ok(())
    }

    fn close_sol(ctx: Context<ClosePhysicalTransactionSol>) -> Result<()>{
        match ctx.bumps.get("escrow_account"){
            Some(escrow_seeds) => close_escrow_sol(
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.seller_wallet.to_account_info(),
                &[&[b"orbit_escrow_account", ctx.accounts.phys_transaction.key().as_ref(), &[*escrow_seeds]]],
                100 // todo: 5% to some address
            ),
            None => return err!(PhysicalMarketErrors::InvalidEscrowBump)
        }.expect("couldnt close escrow properly");

        match ctx.bumps.get("phys_auth"){
            Some(auth_bump) => post_tx_incrementing(
                ctx.accounts.market_account_program.to_account_info(),
                ctx.accounts.buyer_account.to_account_info(),
                ctx.accounts.seller_account.to_account_info(),
                ctx.accounts.physical_auth.to_account_info(),
                ctx.accounts.physical_program.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]]
            ),
            None => return err!(PhysicalMarketErrors::InvalidAuthBump)
        }.expect("could not properly invoke market-accounts program");
        
        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Closed;
        Ok(())
    }

    fn close_spl(ctx: Context<ClosePhysicalTransactionSpl>) -> Result<()>{
        match ctx.bumps.get("phys_auth"){
            Some(auth_bump) => {
                close_escrow_spl(
                    ctx.accounts.token_program.to_account_info(),
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.seller_token_account.to_account_info(),
                    ctx.accounts.physical_auth.to_account_info(),
                    &[&[b"market_authority", &[*auth_bump]]],
                    ctx.accounts.phys_transaction.metadata.transaction_price,
                    100
                )
            },
            None => return err!(PhysicalMarketErrors::InvalidAuthBump)
        }.expect("could not properly invoke market-accounts program");

        match ctx.bumps.get("phys_auth"){
            Some(auth_bump) => post_tx_incrementing(
                ctx.accounts.market_account_program.to_account_info(),
                ctx.accounts.buyer_account.to_account_info(),
                ctx.accounts.seller_account.to_account_info(),
                ctx.accounts.physical_auth.to_account_info(),
                ctx.accounts.physical_program.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]]
            ),
            None => return err!(PhysicalMarketErrors::InvalidAuthBump)
        }.expect("could not properly invoke market-accounts program");

        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Closed;
        Ok(())
    }

    fn fund_escrow_sol(ctx: Context<FundEscrowSol>) -> Result<()>{
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

    fn fund_escrow_spl(ctx: Context<FundEscrowSpl>) -> Result<()>{
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(), 
                anchor_spl::token::Transfer{
                    from: ctx.accounts.buyer_spl_wallet.to_account_info(),
                    to: ctx.accounts.escrow_account.to_account_info(),
                    authority: ctx.accounts.wallet_owner.to_account_info()
                }
            ),
            ctx.accounts.phys_transaction.metadata.transaction_price
        ).expect("could not fund escrow account. maybe check your balance");
        ctx.accounts.phys_transaction.metadata.funded = true;
        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::BuyerFunded;
        Ok(())
    }

    fn close_transaction_account(ctx: Context<CloseTransactionAccount>) -> Result<()>{
        ctx.accounts.phys_transaction.close(ctx.accounts.market_account.to_account_info())
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
    )]
    pub opener: Account<'info, OrbitMarketAccount>,

    #[account(
        address = opener.master_pubkey
    )]
    pub opener_auth: Signer<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,

    pub dispute_program: Program<'info, Dispute>,

    #[account(
        address = phys_transaction.metadata.buyer
    )]
    pub buyer: Account<'info, OrbitMarketAccount>,

    #[account(
        address = phys_transaction.metadata.seller
    )]
    pub seller: Account<'info, OrbitMarketAccount>,

    #[account(
        address = id()
    )]
    /// CHECK: can't use program struct
    pub physical_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>
}

impl<'a, 'b, 'c> OrbitDisputableTrait<'a, 'b, 'c, OpenPhysicalDispute<'a>, ClosePhysicalDisputeSol<'b>, ClosePhysicalDisputeSpl<'c>> for PhysicalTransaction{
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
                        caller_auth: ctx.accounts.physical_auth.to_account_info(),
                        caller_program: ctx.accounts.physical_program.to_account_info(),
                        buyer: ctx.accounts.buyer.to_account_info(),
                        seller: ctx.accounts.seller.to_account_info(),
                        payer: ctx.accounts.payer.to_account_info(),
                        system_program: ctx.accounts.system_program.to_account_info()
                    },
                    &[&[b"physical_auth", &[*signer_bump]]]
                ),
                threshold
            ),
            None => return err!(PhysicalMarketErrors::InvalidAuthBump)
        };

        if res.is_err(){return res};
        Ok(())
    }
    
    fn close_dispute_sol(ctx: Context<ClosePhysicalDisputeSol>) -> Result<()>{
        match ctx.bumps.get("physical_auth"){
            Some(auth_bump) => 
            close_dispute_helper(
                ctx.accounts.dispute_program.to_account_info(),
                ctx.accounts.phys_dispute.to_account_info(),
                ctx.accounts.funder.to_account_info(),
                ctx.accounts.physical_auth.to_account_info(),
                ctx.accounts.physical_program.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]]
            ),
            None => return err!(PhysicalMarketErrors::InvalidAuthBump)
        }.expect("something went wrong");


        match ctx.bumps.get("escrow_account"){
            Some(escrow_bump) => 
            close_escrow_sol(
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.favor.to_account_info(),
                &[&[b"orbit_escrow_account", ctx.accounts.phys_transaction.key().as_ref(), &[*escrow_bump]]],
                100
            ),
            None => return err!(PhysicalMarketErrors::InvalidEscrowBump)
        }.expect("something went wrong");

        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Closed;
        Ok(())
    }

    fn close_dispute_spl(ctx: Context<ClosePhysicalDisputeSpl>) -> Result<()>{
        match ctx.bumps.get("physical_auth"){
            Some(auth_bump) => 
            close_dispute_helper(
                ctx.accounts.dispute_program.to_account_info(),
                ctx.accounts.phys_dispute.to_account_info(),
                ctx.accounts.funder.to_account_info(),
                ctx.accounts.physical_auth.to_account_info(),
                ctx.accounts.physical_program.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]]
            ),
            None => return err!(PhysicalMarketErrors::InvalidAuthBump)
        }.expect("something went wrong");


        match ctx.bumps.get("physical_auth"){
            Some(auth_bump) => 
            close_escrow_spl(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.favor_token_account.to_account_info(),
                ctx.accounts.physical_auth.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]],
                ctx.accounts.phys_transaction.metadata.transaction_price,
                100
            ),
            None => return err!(PhysicalMarketErrors::InvalidEscrowBump)
        }.expect("something went wrong");

        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Closed;
        Ok(())
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

/////////////////////////////////////////////////////////////////////////////////////////////
/// ACCOUNT HELPERS (leave a review)

/// CHECK: the transaction cant be closed as it holds important metadata

#[derive(Accounts)]
pub struct LeaveReview<'info>{
    #[account(mut)]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        mut,
        constraint = 
        (reviewer.key() == phys_transaction.metadata.seller) ||
        (reviewer.key() == phys_transaction.metadata.buyer)
    )]
    pub reviewed_account: Account<'info, OrbitMarketAccount>,

    #[account(
        constraint = 
        (reviewer.key() == phys_transaction.metadata.seller) ||
        (reviewer.key() == phys_transaction.metadata.buyer),
        has_one = master_pubkey
    )]
    pub reviewer: Account<'info, OrbitMarketAccount>,

    pub master_pubkey: Signer<'info>,

    pub accounts_program: Program<'info, OrbitMarketAccounts>,

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub phys_auth: SystemAccount<'info>,

    pub physical_program: Program<'info, OrbitPhysicalMarket>
}

impl <'a> OrbitMarketAccountTrait<'a, LeaveReview<'a>> for PhysicalTransaction{
 
    fn leave_review(ctx: Context<LeaveReview>, rating: u8) -> Result<()>{
        if ctx.accounts.reviewer.key() == ctx.accounts.reviewed_account.key(){
            return err!(ReviewErrors::InvalidReviewAuthority)
        };
        if rating == 0 || rating > 5{
            return err!(ReviewErrors::InvalidReviewAuthority)
        };

        if ctx.accounts.phys_transaction.metadata.seller == ctx.accounts.reviewer.key() && !ctx.accounts.phys_transaction.reviews.seller{
            match ctx.bumps.get("phys_auth"){
                Some(auth_bump) => {
                    submit_rating_with_signer(
                        ctx.accounts.accounts_program.to_account_info(),
                        ctx.accounts.reviewed_account.to_account_info(),
                        ctx.accounts.phys_auth.to_account_info(),
                        ctx.accounts.physical_program.to_account_info(),
                        &[&[b"market_authority", &[*auth_bump]]],
                        rating
                    );
                    ctx.accounts.phys_transaction.reviews.buyer = true;
                },
                None => return err!(MarketAccountErrors::CannotCallOrbitAccountsProgram)
            }
            ctx.accounts.phys_transaction.reviews.seller = true;
        }else
        if ctx.accounts.phys_transaction.metadata.buyer == ctx.accounts.reviewer.key()  && !ctx.accounts.phys_transaction.reviews.buyer{
            match ctx.bumps.get("phys_auth"){
                Some(auth_bump) => {
                    submit_rating_with_signer(
                        ctx.accounts.accounts_program.to_account_info(),
                        ctx.accounts.reviewed_account.to_account_info(),
                        ctx.accounts.phys_auth.to_account_info(),
                        ctx.accounts.physical_program.to_account_info(),
                        &[&[b"market_authority", &[*auth_bump]]],
                        rating
                    );
                    ctx.accounts.phys_transaction.reviews.buyer = true;
                },
                None => return err!(MarketAccountErrors::CannotCallOrbitAccountsProgram)
            }
            ctx.accounts.phys_transaction.reviews.buyer = true;
        }else
        {
            return err!(ReviewErrors::InvalidReviewAuthority)
        };

        Ok(())
    }

}
