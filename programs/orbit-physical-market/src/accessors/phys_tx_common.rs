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
        OrbitMarketAccountTrait
    },
    program::OrbitMarketAccounts,
    MarketAccountErrors
};
use transaction::{
    transaction_errors::ReviewErrors,
    transaction_struct::TransactionState,
    transaction_trait::OrbitTransactionTrait,
    transaction_utils::*
};
use crate::{
    PhysicalTransaction,
    errors::PhysicalMarketErrors,
    
    OpenPhysicalTransactionSol,
    ClosePhysicalTransactionSol,
    FundEscrowSol,
    ClosePhysicalDisputeSol,

    OpenPhysicalTransactionSpl,
    ClosePhysicalTransactionSpl,
    FundEscrowSpl,
    ClosePhysicalDisputeSpl,

    id, program::OrbitPhysicalMarket
};
use orbit_dispute::{
    structs::dispute_trait::OrbitDisputableTrait,
    program::Dispute,
    cpi::accounts::{
        OpenDispute,
        CloseDispute
    }
};
use anchor_spl::token::accessor::amount;

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
            (market_account.voter_id == phys_transaction.metadata.buyer) ||
            (market_account.voter_id == phys_transaction.metadata.seller),
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub market_account:Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        address = market_account.wallet
    )]
    pub wallet: Signer<'info>,

    #[account(
        constraint = buyer_account.voter_id == phys_transaction.metadata.buyer,
        seeds = [
            b"orbit_account",
            buyer_wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub buyer_account:Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>
}

impl<'a, 'b, 'c, 'd, 'e, 'f, 'g> OrbitTransactionTrait<'a, 'b, 'c, 'd, 'e, 'f, 'g, OpenPhysicalTransactionSol<'a>, OpenPhysicalTransactionSpl<'b>, ClosePhysicalTransactionSol<'c>, ClosePhysicalTransactionSpl<'d>, FundEscrowSol<'e>, FundEscrowSpl<'f>, CloseTransactionAccount<'g>> for PhysicalTransaction{
    fn open_sol(ctx: Context<OpenPhysicalTransactionSol>, mut price: u64, use_discount: bool) -> Result<()>{
        if use_discount && ctx.accounts.buyer_account.dispute_discounts > 0{
            ctx.accounts.phys_transaction.metadata.rate = 100;
            price = price * 95 / 100;
            ctx.accounts.buyer_account.dispute_discounts -= 1;
        }else{
            ctx.accounts.phys_transaction.metadata.rate = 95
        }

        ctx.accounts.phys_transaction.metadata.buyer = ctx.accounts.buyer_account.voter_id;
        ctx.accounts.phys_transaction.metadata.seller = ctx.accounts.seller_account.voter_id;
        ctx.accounts.phys_transaction.metadata.product = ctx.accounts.phys_product.key();
        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Opened;
        ctx.accounts.phys_transaction.metadata.transaction_price = price;
        ctx.accounts.phys_transaction.metadata.currency = ctx.accounts.phys_product.metadata.currency;

        ctx.accounts.phys_transaction.metadata.funded = false;

        ctx.accounts.phys_transaction.metadata.escrow_account = ctx.accounts.escrow_account.key();
        ctx.accounts.phys_product.quantity -= 1;
        Ok(())
    }

    fn open_spl(ctx: Context<OpenPhysicalTransactionSpl>, mut price: u64, use_discount: bool) -> Result<()>{
        if use_discount && ctx.accounts.buyer_account.dispute_discounts > 0{
            ctx.accounts.phys_transaction.metadata.rate = 100;
            price = price * 95 / 100;
            ctx.accounts.buyer_account.dispute_discounts -= 1;
        }else{
            ctx.accounts.phys_transaction.metadata.rate = 95
        }
        
        ctx.accounts.phys_transaction.metadata.buyer = ctx.accounts.buyer_account.voter_id;
        ctx.accounts.phys_transaction.metadata.seller = ctx.accounts.seller_account.voter_id;
        ctx.accounts.phys_transaction.metadata.product = ctx.accounts.phys_product.key();
        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Opened;
        ctx.accounts.phys_transaction.metadata.transaction_price = price;
        ctx.accounts.phys_transaction.metadata.currency = ctx.accounts.phys_product.metadata.currency;

        ctx.accounts.phys_transaction.metadata.funded = false;

        ctx.accounts.phys_transaction.metadata.escrow_account = ctx.accounts.escrow_account.key();
        ctx.accounts.phys_product.quantity -= 1;
        Ok(())
    }

    fn close_sol(ctx: Context<'_, '_, '_, 'c, ClosePhysicalTransactionSol<'c>>) -> Result<()>{

        match ctx.bumps.get("escrow_account"){
            Some(escrow_seeds) => {
                if ctx.accounts.phys_transaction.metadata.rate == 95{
                    let bal = ctx.accounts.escrow_account.lamports();
                    let mut residual_amt = bal * 5/100;
                    if  (ctx.accounts.buyer_account.reflink != Pubkey::new(&[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])) &&
                        (ctx.remaining_accounts[0].key() == ctx.accounts.buyer_account.reflink)
                    {
                        let reflink_amt = bal * 25 / 10000;
                        residual_amt = bal * 45/1000;
                        close_escrow_sol_flat(
                            ctx.accounts.escrow_account.to_account_info(),
                            ctx.accounts.buyer_wallet.to_account_info(),
                            &[&[b"orbit_escrow_account", ctx.accounts.phys_transaction.key().as_ref(), &[*escrow_seeds]]],
                            reflink_amt
                        ).expect("couldnt close escrow");
                        
                        match remaining_accounts_to_wallet(ctx.remaining_accounts){
                            Ok(reflink_wallet) => {
                                close_escrow_sol_flat(
                                    ctx.accounts.escrow_account.to_account_info(),
                                    reflink_wallet.to_account_info(),
                                    &[&[b"orbit_escrow_account", ctx.accounts.phys_transaction.key().as_ref(), &[*escrow_seeds]]],
                                    reflink_amt
                                ).expect("couldnt close escrow");
                                reflink_wallet.exit(ctx.program_id)?;
                            },
                            Err(e) => return Err(e)
                        }
                    }
                    close_escrow_sol_flat(
                        ctx.accounts.escrow_account.to_account_info(),
                        ctx.accounts.multisig_wallet.to_account_info(),
                        &[&[b"orbit_escrow_account", ctx.accounts.phys_transaction.key().as_ref(), &[*escrow_seeds]]],
                        residual_amt
                    ).expect("couldnt close escrow");
                }
                
                close_escrow_sol_rate(
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.seller_wallet.to_account_info(),
                    &[&[b"orbit_escrow_account", ctx.accounts.phys_transaction.key().as_ref(), &[*escrow_seeds]]],
                    100
                )
            },
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

    fn close_spl(ctx: Context<'_, '_, '_, 'd, ClosePhysicalTransactionSpl<'d>>) -> Result<()>{
        match ctx.bumps.get("phys_auth"){
            Some(auth_bump) => {

                if ctx.accounts.phys_transaction.metadata.rate == 95{
                    let bal = amount(&ctx.accounts.escrow_account.to_account_info()).expect("could not deserialize token account");
                    let mut residual_amt = bal * 5/100;
                    if  (ctx.accounts.buyer_account.reflink != Pubkey::new(&[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])) &&
                        (ctx.remaining_accounts[0].key() == ctx.accounts.buyer_account.reflink)
                    {
                        let reflink_amt = bal * 25 / 10000;
                        residual_amt = bal * 45/1000;
                        close_escrow_spl_flat(
                            ctx.accounts.token_program.to_account_info(),
                            ctx.accounts.escrow_account.to_account_info(),
                            ctx.accounts.buyer_token_account.to_account_info(),
                            ctx.accounts.physical_auth.to_account_info(),
                            &[&[b"market_authority", &[*auth_bump]]],
                            reflink_amt
                        ).expect("couldnt close escrow");

                        match remaining_accounts_to_token_account(ctx.remaining_accounts){
                            Ok(reflink_token_account) => {
                                close_escrow_spl_flat(
                                    ctx.accounts.token_program.to_account_info(),
                                    ctx.accounts.escrow_account.to_account_info(),
                                    reflink_token_account.to_account_info(),
                                    ctx.accounts.physical_auth.to_account_info(),
                                    &[&[b"market_authority", &[*auth_bump]]],
                                    reflink_amt
                                ).expect("couldnt close escrow");
                                reflink_token_account.exit(ctx.program_id)?;
                            },
                            Err(e) => return Err(e)
                        }
                        
                    }
                    close_escrow_spl_flat(
                        ctx.accounts.token_program.to_account_info(),
                        ctx.accounts.escrow_account.to_account_info(),
                        ctx.accounts.multisig_ata.to_account_info(),
                        ctx.accounts.physical_auth.to_account_info(),
                        &[&[b"market_authority", &[*auth_bump]]],
                        residual_amt
                    ).expect("couldnt close escrow");
                }
                
                close_escrow_spl_rate(
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
                    authority: ctx.accounts.buyer_wallet.to_account_info()
                }
            ),
            ctx.accounts.phys_transaction.metadata.transaction_price
        ).expect("could not fund escrow account. maybe check your balance");
        ctx.accounts.phys_transaction.metadata.funded = true;
        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::BuyerFunded;
        Ok(())
    }

    fn close_transaction_account(ctx: Context<CloseTransactionAccount>) -> Result<()>{
        ctx.accounts.phys_transaction.close(ctx.accounts.buyer_account.to_account_info())
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
        seeds::program = orbit_dispute::ID,
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
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,

    #[account(
        mut,
        constraint = (opener_wallet.key() == buyer.wallet) || (opener_wallet.key() == seller.wallet),
    )]
    pub opener_wallet: Signer<'info>,

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,

    pub dispute_program: Program<'info, Dispute>,

    #[account(
        constraint = buyer.voter_id == phys_transaction.metadata.buyer
    )]
    pub buyer:Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        constraint = seller.voter_id == phys_transaction.metadata.seller
    )]
    pub seller:Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        address = id()
    )]
    /// CHECK: can't use program struct
    pub physical_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>
}

pub fn close_dispute_helper<'a>(dispute_program: AccountInfo<'a>, dispute_struct: AccountInfo<'a>, funder: AccountInfo<'a>, program_auth: AccountInfo<'a>, program: AccountInfo<'a>, seeds: &[&[&[u8]]]) -> Result<()>{
    orbit_dispute::cpi::close_dispute(
        CpiContext::new_with_signer(
            dispute_program,
            CloseDispute{
                dispute_account: dispute_struct,
                funder: funder,
                caller_auth: program_auth,
                caller: program
            },
            seeds
        )
    )
}

impl<'a, 'b, 'c> OrbitDisputableTrait<'a, 'b, 'c, OpenPhysicalDispute<'a>, ClosePhysicalDisputeSol<'b>, ClosePhysicalDisputeSpl<'c>> for PhysicalTransaction{
    fn open_dispute(ctx: Context<OpenPhysicalDispute>, threshold: u8) -> Result<()>{
        if (!ctx.accounts.new_dispute.data_is_empty()) || (ctx.accounts.new_dispute.lamports() > 0){
            return err!(PhysicalMarketErrors::DisputeExists)
        }

        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Frozen;

        let res: Result<()> = match ctx.bumps.get("physical_auth"){
            Some(signer_bump) => orbit_dispute::cpi::open_dispute(
                CpiContext::new_with_signer(
                    ctx.accounts.dispute_program.to_account_info(),
                    OpenDispute{
                        new_dispute: ctx.accounts.new_dispute.to_account_info(),
                        in_transaction: ctx.accounts.phys_transaction.to_account_info(),
                        caller_auth: ctx.accounts.physical_auth.to_account_info(),
                        caller_program: ctx.accounts.physical_program.to_account_info(),
                        buyer: ctx.accounts.buyer.to_account_info(),
                        seller: ctx.accounts.seller.to_account_info(),
                        payer: ctx.accounts.opener_wallet.to_account_info(),
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

        match ctx.bumps.get("escrow_account"){
            Some(escrow_bump) => {
                // no reflink or discount option for disputes
                if ctx.accounts.phys_transaction.metadata.rate == 95{
                    close_escrow_sol_rate(
                        ctx.accounts.escrow_account.to_account_info(),
                        ctx.accounts.multisig_wallet.to_account_info(),
                        &[&[b"orbit_escrow_account", ctx.accounts.phys_transaction.key().as_ref(), &[*escrow_bump]]],
                        100 - ctx.accounts.phys_transaction.metadata.rate
                    ).expect("couldnt close escrow");
                }
                close_escrow_sol_rate(
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.favor_wallet.to_account_info(),
                    &[&[b"orbit_escrow_account", ctx.accounts.phys_transaction.key().as_ref(), &[*escrow_bump]]],
                    100
                )
            },
            None => return err!(PhysicalMarketErrors::InvalidEscrowBump)
        }.expect("something went wrong");

        if ctx.accounts.phys_transaction.metadata.rate == 100 && ctx.accounts.favor_market_account.voter_id == ctx.accounts.phys_transaction.metadata.buyer {
            ctx.accounts.favor_market_account.dispute_discounts += 1;
        }

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

        ctx.accounts.phys_transaction.metadata.transaction_state = TransactionState::Closed;
        Ok(())
    }

    fn close_dispute_spl(ctx: Context<ClosePhysicalDisputeSpl>) -> Result<()>{

        match ctx.bumps.get("physical_auth"){
            Some(auth_bump) => {
                close_escrow_spl_rate(
                    ctx.accounts.token_program.to_account_info(),
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.multisig_ata.to_account_info(),
                    ctx.accounts.physical_auth.to_account_info(),
                    &[&[b"market_authority", &[*auth_bump]]],
                    ctx.accounts.phys_transaction.metadata.transaction_price,
                    100-ctx.accounts.phys_transaction.metadata.rate
                ).expect("couldnt close dispute escrow");
                close_escrow_spl_rate(
                    ctx.accounts.token_program.to_account_info(),
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.favor_token_account.to_account_info(),
                    ctx.accounts.physical_auth.to_account_info(),
                    &[&[b"market_authority", &[*auth_bump]]],
                    ctx.accounts.phys_transaction.metadata.transaction_price,
                    100
                )
            },
            None => return err!(PhysicalMarketErrors::InvalidEscrowBump)
        }.expect("something went wrong");

        if ctx.accounts.phys_transaction.metadata.rate == 100 && ctx.accounts.favor_market_account.voter_id == ctx.accounts.phys_transaction.metadata.buyer {
            ctx.accounts.favor_market_account.dispute_discounts += 1;
        }

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
        constraint = seller_account.voter_id == phys_transaction.metadata.seller,
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub seller_account:Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        address = seller_account.wallet
    )]
    pub wallet: Signer<'info>,
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
        constraint = buyer_account.voter_id == phys_transaction.metadata.buyer,
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub buyer_account:Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        address = buyer_account.wallet
    )]
    pub wallet: Signer<'info>,
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

#[derive(Accounts)]
pub struct LeaveReview<'info>{
    #[account(mut)]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        mut,
        constraint = 
        (reviewer.voter_id == phys_transaction.metadata.seller) ||
        (reviewer.voter_id == phys_transaction.metadata.buyer)
    )]
    pub reviewed_account:Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        constraint = 
        (reviewer.voter_id == phys_transaction.metadata.seller) ||
        (reviewer.voter_id == phys_transaction.metadata.buyer),
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub reviewer:Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        address = reviewer.wallet
    )]
    pub wallet: Signer<'info>,

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

        if ctx.accounts.phys_transaction.metadata.seller == ctx.accounts.reviewer.voter_id && !ctx.accounts.phys_transaction.metadata.reviews.seller{
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
                    ctx.accounts.phys_transaction.metadata.reviews.seller = true;
                },
                None => return err!(MarketAccountErrors::CannotCallOrbitAccountsProgram)
            }
        }else
        if ctx.accounts.phys_transaction.metadata.buyer == ctx.accounts.reviewer.voter_id  && !ctx.accounts.phys_transaction.metadata.reviews.buyer{
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
                    ctx.accounts.phys_transaction.metadata.reviews.buyer = true;
                },
                None => return err!(MarketAccountErrors::CannotCallOrbitAccountsProgram)
            }
        }else
        {
            return err!(ReviewErrors::InvalidReviewAuthority)
        };

        Ok(())
    }

}
