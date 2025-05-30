use anchor_lang::{
    prelude::*,
    AccountsClose,
    solana_program::{
        system_instruction::transfer,
        program::{
            invoke,
            invoke_signed
        },
    },
};
use market_accounts::{
    OrbitMarketAccount,
    OrbitMarketAccountTrait,
    OrbitReflink,
    ReviewErrors,
    MarketAccountErrors,
    program::OrbitMarketAccounts,
};
use orbit_transaction::{
    TransactionState,
    OrbitTransactionTrait,
    TransactionErrors, BuyerOpenTransactions, SellerOpenTransactions, TransactionReviews
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

    program::OrbitPhysicalMarket, SellerEarlyDeclineSpl, SellerEarlyDeclineSol
};
use orbit_dispute::{
    structs::dispute_trait::OrbitDisputableTrait,
    program::Dispute,
    cpi::accounts::{
        OpenDispute,
        CloseDispute
    }
};
use anchor_spl::token::{
    accessor::amount,
    TokenAccount
};

////////////////////////////////////////////////////////////////////
/// ORBIT BASE TRANSACTION FUNCTIONALITIES

#[derive(Accounts)]
pub struct CloseTransactionAccount<'info>{
    #[account(
        mut,
        constraint = physical_transaction.metadata.transaction_state == TransactionState::Closed,
    )]
    pub physical_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        has_one = wallet,
        constraint = {
            (proposer_account.voter_id == physical_transaction.metadata.seller) ||
            (proposer_account.voter_id == physical_transaction.metadata.buyer)
        }
    )]
    pub proposer_account: Account<'info, OrbitMarketAccount>,

    pub wallet: Signer<'info>,
    
    #[account(
        constraint = {buyer_account.voter_id == physical_transaction.metadata.buyer}
    )]
    pub buyer_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut
    )]
    pub buyer_wallet: SystemAccount<'info>
}

impl<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i> OrbitTransactionTrait<'a, 'b, 'c, 'd, 'e, 'f, 'g, 'h, 'i, OpenPhysicalTransactionSol<'a>, OpenPhysicalTransactionSpl<'b>, ClosePhysicalTransactionSol<'c>, ClosePhysicalTransactionSpl<'d>, FundEscrowSol<'e>, FundEscrowSpl<'f>, CloseTransactionAccount<'g>, SellerEarlyDeclineSol<'h>, SellerEarlyDeclineSpl<'i>> for PhysicalTransaction{
    fn open_sol(ctx: Context<OpenPhysicalTransactionSol>, seller_index: u8, buyer_index: u8, mut price: u64, use_discount: bool) -> Result<()>{
        
        let auth_bump: &u8;
        if let Some(ab) = ctx.bumps.get("physical_auth"){
            auth_bump = ab
        }else{
            return err!(PhysicalMarketErrors::InvalidAuthBump)
        };

        if use_discount && ctx.accounts.buyer_market_account.dispute_discounts > 0{
            ctx.accounts.physical_transaction.metadata.rate = 100;
            price = price * 95 / 100;
            
            market_accounts::cpi::decrement_dispute_discounts(
                CpiContext::new_with_signer(
                    ctx.accounts.market_account_program.to_account_info(),
                    market_accounts::cpi::accounts::MarketAccountUpdateInternal{
                        market_account: ctx.accounts.buyer_market_account.to_account_info(),
                        caller_auth: ctx.accounts.physical_auth.to_account_info(),
                        caller: ctx.accounts.physical_program.to_account_info()
                    },
                    &[&[b"market_authority", &[*auth_bump]]]
                )
            )?;

        }else{
            ctx.accounts.physical_transaction.metadata.rate = 95
        }

        ctx.accounts.physical_transaction.metadata.buyer = ctx.accounts.buyer_market_account.voter_id;
        ctx.accounts.physical_transaction.metadata.seller = ctx.accounts.seller_market_account.voter_id;
        ctx.accounts.physical_transaction.metadata.product = ctx.accounts.phys_product.metadata.index;
        ctx.accounts.physical_transaction.metadata.transaction_state = TransactionState::Opened;
        ctx.accounts.physical_transaction.metadata.transaction_price = price;
        ctx.accounts.physical_transaction.metadata.currency = System::id();
        ctx.accounts.physical_transaction.metadata.funded = false;
        ctx.accounts.physical_transaction.metadata.buyer_tx_index = buyer_index;
        ctx.accounts.physical_transaction.metadata.seller_tx_index = seller_index;
        ctx.accounts.physical_transaction.metadata.reviews = TransactionReviews{
            buyer: false,
            seller: false
        };

        orbit_product::cpi::update_product_quantity_internal(
            CpiContext::new_with_signer(
                ctx.accounts.product_program.to_account_info(),
                orbit_product::cpi::accounts::UpdatePhysicalQuantityInternal{
                    product: ctx.accounts.phys_product.to_account_info(),
                    vendor_account: ctx.accounts.seller_market_account.to_account_info(),
                    vendor_listings: ctx.accounts.seller_listings.to_account_info(),
                    caller_auth: ctx.accounts.physical_auth.to_account_info(),
                    caller: ctx.accounts.physical_program.to_account_info()
                },
                &[&[b"market_authority", &[*auth_bump]]]
            ),
            ctx.accounts.phys_product.quantity-1
        )?;

        orbit_transaction::cpi::add_buyer_physical_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::AddBuyerPhysicalTransactions{
                    transactions_log: ctx.accounts.buyer_transactions_log.to_account_info(),
                    tx: ctx.accounts.physical_transaction.to_account_info(),
                    buyer_account: ctx.accounts.buyer_market_account.to_account_info(),
                    wallet: ctx.accounts.buyer_wallet.to_account_info(),
                }
            ),
            buyer_index
        )?;
        orbit_transaction::cpi::add_seller_physical_transaction(
            CpiContext::new_with_signer(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::AddSellerPhysicalTransactions{
                    transactions_log: ctx.accounts.seller_transactions_log.to_account_info(),
                    tx: ctx.accounts.physical_transaction.to_account_info()
                },
                &[&[b"market_authority", &[*auth_bump]]]
            ),
            seller_index
        )?;
        Ok(())
    }

    fn open_spl(ctx: Context<OpenPhysicalTransactionSpl>, seller_index: u8, buyer_index: u8, mut price: u64, use_discount: bool) -> Result<()>{
        let auth_bump: &u8;
        if let Some(ab) = ctx.bumps.get("physical_auth"){
            auth_bump = ab
        }else{
            return err!(PhysicalMarketErrors::InvalidAuthBump)
        };
        if use_discount && ctx.accounts.buyer_market_account.dispute_discounts > 0{
            ctx.accounts.physical_transaction.metadata.rate = 100;
            price = price * 95 / 100;
            
            market_accounts::cpi::decrement_dispute_discounts(
                CpiContext::new_with_signer(
                    ctx.accounts.market_account_program.to_account_info(),
                    market_accounts::cpi::accounts::MarketAccountUpdateInternal{
                        market_account: ctx.accounts.buyer_market_account.to_account_info(),
                        caller_auth: ctx.accounts.physical_auth.to_account_info(),
                        caller: ctx.accounts.physical_program.to_account_info()
                    },
                    &[&[b"market_authority", &[*auth_bump]]]
                )
            )?;

        }else{
            ctx.accounts.physical_transaction.metadata.rate = 95
        };
        
        ctx.accounts.physical_transaction.metadata.buyer = ctx.accounts.buyer_market_account.voter_id;
        ctx.accounts.physical_transaction.metadata.seller = ctx.accounts.seller_market_account.voter_id;
        ctx.accounts.physical_transaction.metadata.product = ctx.accounts.phys_product.metadata.index;
        ctx.accounts.physical_transaction.metadata.transaction_state = TransactionState::Opened;
        ctx.accounts.physical_transaction.metadata.transaction_price = price;
        ctx.accounts.physical_transaction.metadata.currency = ctx.accounts.token_mint.key();
        ctx.accounts.physical_transaction.metadata.buyer_tx_index = buyer_index;
        ctx.accounts.physical_transaction.metadata.seller_tx_index = seller_index;
        ctx.accounts.physical_transaction.metadata.funded = false;
        ctx.accounts.physical_transaction.metadata.reviews = TransactionReviews{
            buyer: false,
            seller: false
        };

        orbit_product::cpi::update_product_quantity_internal(
            CpiContext::new_with_signer(
                ctx.accounts.product_program.to_account_info(),
                orbit_product::cpi::accounts::UpdatePhysicalQuantityInternal{
                    product: ctx.accounts.phys_product.to_account_info(),
                    vendor_account: ctx.accounts.seller_market_account.to_account_info(),
                    vendor_listings: ctx.accounts.seller_listings.to_account_info(),
                    caller_auth: ctx.accounts.physical_auth.to_account_info(),
                    caller: ctx.accounts.physical_program.to_account_info()
                },
                &[&[b"market_authority", &[*auth_bump]]]
            ),
            ctx.accounts.phys_product.quantity-1
        )?;

        orbit_transaction::cpi::add_buyer_physical_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::AddBuyerPhysicalTransactions{
                    transactions_log: ctx.accounts.buyer_transactions_log.to_account_info(),
                    tx: ctx.accounts.physical_transaction.to_account_info(),
                    buyer_account: ctx.accounts.buyer_market_account.to_account_info(),
                    wallet: ctx.accounts.buyer_wallet.to_account_info(),
                }
            ),
            buyer_index
        )?;
        orbit_transaction::cpi::add_seller_physical_transaction(
            CpiContext::new_with_signer(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::AddSellerPhysicalTransactions{
                    transactions_log: ctx.accounts.seller_transactions_log.to_account_info(),
                    tx: ctx.accounts.physical_transaction.to_account_info()
                },
                &[&[b"market_authority", &[*auth_bump]]]
            ),
            seller_index
        )?;
        
        Ok(())
    }

    fn close_sol(ctx: Context<'_, '_, '_, 'c, ClosePhysicalTransactionSol<'c>>) -> Result<()>{
        let physical_tx = ctx.accounts.physical_transaction.key();
        let physical_seed = physical_tx.as_ref();
        let buyer_tx_log = ctx.accounts.buyer_account.key();
        let buyer_tx_log_seed = buyer_tx_log.as_ref();

        if let Some(escrow_seeds) = ctx.bumps.get("escrow_account"){
            if ctx.accounts.physical_transaction.metadata.rate == 95{
                let bal = ctx.accounts.escrow_account.lamports();
                let mut residual_amt = bal * 5/100;
                if  (ctx.accounts.buyer_account.used_reflink != Pubkey::from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])) &&
                    (ctx.remaining_accounts[0].key() == ctx.accounts.buyer_account.used_reflink)
                {
                    let reflink_amt = bal * 25 / 10000;
                    residual_amt = bal * 45/1000;
                    orbit_transaction::close_escrow_sol_flat!(
                        ctx.accounts.escrow_account.to_account_info(),
                        ctx.accounts.buyer_wallet.to_account_info(),
                        &[&[b"orbit_escrow_account", physical_seed, buyer_tx_log_seed, &[*escrow_seeds]]],
                        reflink_amt
                    ).expect("couldnt close escrow");
                    
                    match orbit_transaction::remaining_accounts_to_wallet!(ctx.remaining_accounts){
                        Ok(reflink_wallet) => {
                            orbit_transaction::close_escrow_sol_flat!(
                                ctx.accounts.escrow_account.to_account_info(),
                                reflink_wallet.to_account_info(),
                                &[&[b"orbit_escrow_account", physical_seed, buyer_tx_log_seed, &[*escrow_seeds]]],
                                reflink_amt
                            ).expect("couldnt close escrow");
                            reflink_wallet.exit(ctx.program_id)?;
                        },
                        Err(e) => return Err(e)
                    }
                }
                orbit_transaction::close_escrow_sol_flat!(
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.multisig_wallet.to_account_info(),
                    &[&[b"orbit_escrow_account", physical_seed, buyer_tx_log_seed, &[*escrow_seeds]]],
                    residual_amt
                ).expect("couldnt close escrow");
            };
            
            orbit_transaction::close_escrow_sol_rate!(
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.seller_wallet.to_account_info(),
                &[&[b"orbit_escrow_account", physical_seed, buyer_tx_log_seed, &[*escrow_seeds]]],
                100
            )?;
        }else{
            return err!(PhysicalMarketErrors::InvalidEscrowBump)
        };

        if let Some(auth_bump) = ctx.bumps.get("phys_auth"){
            orbit_transaction::post_tx_incrementing!(
                ctx.accounts.market_account_program.to_account_info(),
                ctx.accounts.buyer_account.to_account_info(),
                ctx.accounts.seller_account.to_account_info(),
                ctx.accounts.physical_auth.to_account_info(),
                ctx.accounts.physical_program.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]]
            )?;

            orbit_product::cpi::physical_increment_times_sold(
                CpiContext::new_with_signer(
                    ctx.accounts.product_program.to_account_info(),
                    orbit_product::cpi::accounts::IncrementPhysicalSoldInternal{
                        product: ctx.accounts.phys_product.to_account_info(),
                        caller_auth: ctx.accounts.physical_auth.to_account_info(),
                        caller: ctx.accounts.physical_program.to_account_info()
                    },
                    &[&[b"market_authority", &[*auth_bump]]]
                )
            )?;
        }else{
            return err!(PhysicalMarketErrors::InvalidAuthBump)
        };

        orbit_transaction::cpi::clear_seller_physical_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearSellerPhysicalTransactions{
                    transactions_log: ctx.accounts.seller_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.physical_auth.to_account_info(),
                    caller: ctx.accounts.physical_program.to_account_info()
                }
            ),
            ctx.accounts.physical_transaction.metadata.seller_tx_index
        )?;

        orbit_transaction::cpi::clear_buyer_physical_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearBuyerPhysicalTransactions{
                    transactions_log: ctx.accounts.buyer_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.physical_auth.to_account_info(),
                    caller: ctx.accounts.physical_program.to_account_info()
                }
            ),
            ctx.accounts.physical_transaction.metadata.seller_tx_index
        )?;
        
        ctx.accounts.physical_transaction.metadata.transaction_state = TransactionState::Closed;
        Ok(())
    }

    fn close_spl(ctx: Context<'_, '_, '_, 'd, ClosePhysicalTransactionSpl<'d>>) -> Result<()>{
        if let Some(auth_bump) = ctx.bumps.get("phys_auth"){
            if ctx.accounts.physical_transaction.metadata.rate == 95{
                let bal = amount(&ctx.accounts.escrow_account.to_account_info()).expect("could not deserialize token account");
                let mut residual_amt = bal * 5/100;
                if  (ctx.accounts.buyer_account.used_reflink != Pubkey::from([0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0])) &&
                    (ctx.remaining_accounts[0].key() == ctx.accounts.buyer_account.used_reflink)
                {
                    let reflink_amt = bal * 25 / 10000;
                    residual_amt = bal * 45/1000;
                    orbit_transaction::close_escrow_spl_flat!(
                        ctx.accounts.token_program.to_account_info(),
                        ctx.accounts.escrow_account.to_account_info(),
                        ctx.accounts.buyer_token_account.to_account_info(),
                        ctx.accounts.physical_auth.to_account_info(),
                        &[&[b"market_authority", &[*auth_bump]]],
                        reflink_amt
                    ).expect("couldnt close escrow");

                    match orbit_transaction::remaining_accounts_to_token_account!(ctx.remaining_accounts){
                        Ok(reflink_token_account) => {
                            orbit_transaction::close_escrow_spl_flat!(
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
                orbit_transaction::close_escrow_spl_flat!(
                    ctx.accounts.token_program.to_account_info(),
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.multisig_ata.to_account_info(),
                    ctx.accounts.physical_auth.to_account_info(),
                    &[&[b"market_authority", &[*auth_bump]]],
                    residual_amt
                ).expect("couldnt close escrow");
            }
            
            orbit_transaction::post_tx_incrementing!(
                ctx.accounts.market_account_program.to_account_info(),
                ctx.accounts.buyer_account.to_account_info(),
                ctx.accounts.seller_account.to_account_info(),
                ctx.accounts.physical_auth.to_account_info(),
                ctx.accounts.physical_program.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]]
            )?;
            orbit_product::cpi::physical_increment_times_sold(
                CpiContext::new_with_signer(
                    ctx.accounts.product_program.to_account_info(),
                    orbit_product::cpi::accounts::IncrementPhysicalSoldInternal{
                        product: ctx.accounts.phys_product.to_account_info(),
                        caller_auth: ctx.accounts.physical_auth.to_account_info(),
                        caller: ctx.accounts.physical_program.to_account_info()
                    },
                    &[&[b"market_authority", &[*auth_bump]]]
                )
            )?;
            
            orbit_transaction::close_escrow_spl_rate!(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.seller_token_account.to_account_info(),
                ctx.accounts.physical_auth.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]],
                ctx.accounts.physical_transaction.metadata.transaction_price,
                100
            )
        }else{
            return err!(PhysicalMarketErrors::InvalidAuthBump)
        }?;

        orbit_transaction::cpi::clear_seller_physical_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearSellerPhysicalTransactions{
                    transactions_log: ctx.accounts.seller_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.physical_auth.to_account_info(),
                    caller: ctx.accounts.physical_program.to_account_info()
                }
            ),
            ctx.accounts.physical_transaction.metadata.seller_tx_index
        )?;

        orbit_transaction::cpi::clear_buyer_physical_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearBuyerPhysicalTransactions{
                    transactions_log: ctx.accounts.buyer_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.physical_auth.to_account_info(),
                    caller: ctx.accounts.physical_program.to_account_info()
                }
            ),
            ctx.accounts.physical_transaction.metadata.seller_tx_index
        )?;

        ctx.accounts.physical_transaction.metadata.transaction_state = TransactionState::Closed;
        Ok(())
    }

    fn fund_escrow_sol(ctx: Context<FundEscrowSol>) -> Result<()>{
        invoke(
            &transfer(
                &ctx.accounts.buyer_wallet.key(),
                &ctx.accounts.escrow_account.key(),
                ctx.accounts.physical_transaction.metadata.transaction_price
            ),
            &[
                ctx.accounts.buyer_wallet.to_account_info(),
                ctx.accounts.escrow_account.to_account_info()
            ]
        ).expect("could not fund escrow");
        ctx.accounts.physical_transaction.metadata.funded = true;
        ctx.accounts.physical_transaction.metadata.transaction_state = TransactionState::BuyerFunded;
        Ok(())
    }

    fn fund_escrow_spl(ctx: Context<FundEscrowSpl>) -> Result<()>{
        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(), 
                anchor_spl::token::Transfer{
                    from: ctx.accounts.buyer_token_account.to_account_info(),
                    to: ctx.accounts.escrow_account.to_account_info(),
                    authority: ctx.accounts.buyer_wallet.to_account_info()
                }
            ),
            ctx.accounts.physical_transaction.metadata.transaction_price
        ).expect("could not fund escrow account. maybe check your balance");
        ctx.accounts.physical_transaction.metadata.funded = true;
        ctx.accounts.physical_transaction.metadata.transaction_state = TransactionState::BuyerFunded;
        Ok(())
    }

    fn close_transaction_account(ctx: Context<CloseTransactionAccount>) -> Result<()>{
        ctx.accounts.physical_transaction.close(ctx.accounts.buyer_wallet.to_account_info())
    }
    
    fn seller_early_decline_sol(ctx: Context<SellerEarlyDeclineSol>) -> Result<()>{
        ctx.accounts.physical_transaction.metadata.transaction_state = TransactionState::Closed;
        if ctx.accounts.physical_transaction.metadata.rate == 100{
            market_accounts::cpi::increment_dispute_discounts(
                CpiContext::new(
                    ctx.accounts.market_account_program.to_account_info(),
                    market_accounts::cpi::accounts::MarketAccountUpdateInternal{
                        market_account: ctx.accounts.buyer_account.to_account_info(),
                        caller_auth: ctx.accounts.physical_auth.to_account_info(),
                        caller: ctx.accounts.physical_program.to_account_info()
                    }
                )
            )?;
        };
        
        let physical_tx = ctx.accounts.physical_transaction.key();
        let physical_seed = physical_tx.as_ref();
        let buyer_tx_log = ctx.accounts.buyer_account.key();
        let buyer_tx_log_seed = buyer_tx_log.as_ref();

        if let Some(escrow_seeds) = ctx.bumps.get("escrow_account"){
            orbit_transaction::close_escrow_sol_rate!(
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.buyer_wallet.to_account_info(),
                &[&[b"orbit_escrow_account", physical_seed, buyer_tx_log_seed, &[*escrow_seeds]]],
                100
            )?;
        }else{
            return err!(PhysicalMarketErrors::InvalidEscrowBump)
        };
        
        orbit_transaction::cpi::clear_seller_physical_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearSellerPhysicalTransactions{
                    transactions_log: ctx.accounts.seller_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.physical_auth.to_account_info(),
                    caller: ctx.accounts.physical_program.to_account_info()
                }
            ),
            ctx.accounts.physical_transaction.metadata.seller_tx_index
        )?;

        orbit_transaction::cpi::clear_buyer_physical_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearBuyerPhysicalTransactions{
                    transactions_log: ctx.accounts.buyer_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.physical_auth.to_account_info(),
                    caller: ctx.accounts.physical_program.to_account_info()
                }
            ),
            ctx.accounts.physical_transaction.metadata.seller_tx_index
        )?;
        Ok(())
    }

    fn seller_early_decline_spl(ctx: Context<SellerEarlyDeclineSpl>) -> Result<()>{
        ctx.accounts.physical_transaction.metadata.transaction_state = TransactionState::Closed;

        if ctx.accounts.physical_transaction.metadata.rate == 100{
            market_accounts::cpi::increment_dispute_discounts(
                CpiContext::new(
                    ctx.accounts.market_account_program.to_account_info(),
                    market_accounts::cpi::accounts::MarketAccountUpdateInternal{
                        market_account: ctx.accounts.buyer_market_account.to_account_info(),
                        caller_auth: ctx.accounts.physical_auth.to_account_info(),
                        caller: ctx.accounts.physical_program.to_account_info()
                    }
                )
            )?;
        }

        if let Some(auth_bump) = ctx.bumps.get("phys_auth"){
            orbit_transaction::close_escrow_spl_rate!(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.buyer_token_account.to_account_info(),
                ctx.accounts.physical_auth.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]],
                ctx.accounts.physical_transaction.metadata.transaction_price,
                100
            )
        }else{
            return err!(PhysicalMarketErrors::InvalidAuthBump)
        }?;    
        
        orbit_transaction::cpi::clear_seller_physical_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearSellerPhysicalTransactions{
                    transactions_log: ctx.accounts.seller_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.physical_auth.to_account_info(),
                    caller: ctx.accounts.physical_program.to_account_info()
                }
            ),
            ctx.accounts.physical_transaction.metadata.seller_tx_index
        )?;

        orbit_transaction::cpi::clear_buyer_physical_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearBuyerPhysicalTransactions{
                    transactions_log: ctx.accounts.buyer_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.physical_auth.to_account_info(),
                    caller: ctx.accounts.physical_program.to_account_info()
                }
            ),
            ctx.accounts.physical_transaction.metadata.seller_tx_index
        )?;

        Ok(())

    }
}

////////////////////////////////////////////////////////////////////
/// ORBIT DISPUTE FUNCTIONALITIES
#[derive(Accounts)]
pub struct OpenPhysicalDispute<'info>{
    ////////////////////////////////////////////
    /// TX
    #[account(
        mut,
        constraint =
        (physical_transaction.metadata.transaction_state == TransactionState::BuyerFunded) ||
        (physical_transaction.metadata.transaction_state == TransactionState::Shipped) ||
        (physical_transaction.metadata.transaction_state == TransactionState::BuyerConfirmedDelivery)
    )]
    pub physical_transaction: Box<Account<'info, PhysicalTransaction>>,

    ////////////////////////////////////////////
    /// DISPUTE
    #[account(
        mut,
        seeds = [
            b"dispute_account",
            physical_transaction.key().as_ref()
        ],
        seeds::program = orbit_dispute::ID,
        bump
    )]
    pub new_dispute: SystemAccount<'info>,

    #[account(
        mut,
        constraint = (opener_wallet.key() == buyer.wallet) || (opener_wallet.key() == seller.wallet),
    )]
    pub opener_wallet: Signer<'info>,

    ////////////////////////////////////////////
    /// BUYER SELLER
    
    #[account(
        constraint = buyer.voter_id == physical_transaction.metadata.buyer
    )]
    pub buyer: Box<Account<'info, OrbitMarketAccount>>,
    
    #[account(
        constraint = seller.voter_id == physical_transaction.metadata.seller
    )]
    pub seller: Box<Account<'info, OrbitMarketAccount>>,
    
    /////////////////////////////////
    /// EXTRANEOUS

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,

    pub dispute_program: Program<'info, Dispute>,
    
    pub physical_program: Program<'info, OrbitPhysicalMarket>,
    
    pub system_program: Program<'info, System>
}

pub fn close_dispute_helper<'a>(dispute_program: AccountInfo<'a>, dispute_struct: AccountInfo<'a>, funder: AccountInfo<'a>, program_auth: AccountInfo<'a>, program: AccountInfo<'a>, seeds: &[&[&[u8]]]) -> Result<()>{
    orbit_dispute::cpi::close_dispute(
        CpiContext::new_with_signer(
            dispute_program,
            CloseDispute{
                dispute_account: dispute_struct,
                funder,
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

        ctx.accounts.physical_transaction.metadata.transaction_state = TransactionState::Frozen;

        if let Some(signer_bump) = ctx.bumps.get("physical_auth"){
            orbit_dispute::cpi::open_dispute(
                CpiContext::new_with_signer(
                    ctx.accounts.dispute_program.to_account_info(),
                    OpenDispute{
                        new_dispute: ctx.accounts.new_dispute.to_account_info(),
                        in_transaction: ctx.accounts.physical_transaction.to_account_info(),
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
            ) 
        }else{
            return err!(PhysicalMarketErrors::InvalidAuthBump)
        }?;

        Ok(())
    }
    
    fn close_dispute_sol(ctx: Context<ClosePhysicalDisputeSol>) -> Result<()>{

        let physical_tx = ctx.accounts.physical_transaction.key();
        let physical_seed = physical_tx.as_ref();
        let buyer_tx_log = ctx.accounts.buyer_account.key();
        let buyer_tx_log_seed = buyer_tx_log.as_ref();

        if let Some(escrow_bump) = ctx.bumps.get("escrow_account"){
            if ctx.accounts.physical_transaction.metadata.rate == 95{
                orbit_transaction::close_escrow_sol_rate!(
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.multisig_wallet.to_account_info(),
                    &[&[b"orbit_escrow_account", physical_seed, buyer_tx_log_seed, &[*escrow_bump]]],
                    100 - ctx.accounts.physical_transaction.metadata.rate
                ).expect("couldnt close escrow");
            }
            orbit_transaction::close_escrow_sol_rate!(
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.favor_wallet.to_account_info(),
                &[&[b"orbit_escrow_account", physical_seed, buyer_tx_log_seed, &[*escrow_bump]]],
                100
            )
        }else{
            return err!(PhysicalMarketErrors::InvalidEscrowBump)
        }?;
        
        if ctx.accounts.physical_transaction.metadata.rate == 100 && ctx.accounts.favor_market_account.voter_id == ctx.accounts.physical_transaction.metadata.buyer {
            // todo: whyd i fucking do this
            // ctx.accounts.favor_market_account.dispute_discounts += 1;
        }

        if let Some(auth_bump) = ctx.bumps.get("physical_auth"){
            close_dispute_helper(
                ctx.accounts.dispute_program.to_account_info(),
                ctx.accounts.phys_dispute.to_account_info(),
                ctx.accounts.funder.to_account_info(),
                ctx.accounts.physical_auth.to_account_info(),
                ctx.accounts.physical_program.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]]
            )
        }else{
            return err!(PhysicalMarketErrors::InvalidAuthBump)
        }?;

        orbit_transaction::cpi::clear_seller_physical_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearSellerPhysicalTransactions{
                    transactions_log: ctx.accounts.seller_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.physical_auth.to_account_info(),
                    caller: ctx.accounts.physical_program.to_account_info()
                }
            ),
            ctx.accounts.physical_transaction.metadata.seller_tx_index
        )?;

        orbit_transaction::cpi::clear_buyer_physical_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearBuyerPhysicalTransactions{
                    transactions_log: ctx.accounts.buyer_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.physical_auth.to_account_info(),
                    caller: ctx.accounts.physical_program.to_account_info()
                }
            ),
            ctx.accounts.physical_transaction.metadata.seller_tx_index
        )?;

        ctx.accounts.physical_transaction.metadata.transaction_state = TransactionState::Closed;
        Ok(())
    }

    fn close_dispute_spl(ctx: Context<ClosePhysicalDisputeSpl>) -> Result<()>{
        if let Some(auth_bump) = ctx.bumps.get("physical_auth"){
            orbit_transaction::close_escrow_spl_rate!(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.multisig_ata.to_account_info(),
                ctx.accounts.physical_auth.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]],
                ctx.accounts.physical_transaction.metadata.transaction_price,
                100-ctx.accounts.physical_transaction.metadata.rate
            ).expect("couldnt close dispute escrow");
            orbit_transaction::close_escrow_spl_rate!(
                ctx.accounts.token_program.to_account_info(),
                ctx.accounts.escrow_account.to_account_info(),
                ctx.accounts.favor_token_account.to_account_info(),
                ctx.accounts.physical_auth.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]],
                ctx.accounts.physical_transaction.metadata.transaction_price,
                100
            )
        }else{
            return err!(PhysicalMarketErrors::InvalidEscrowBump)
        }?;

        if ctx.accounts.physical_transaction.metadata.rate == 100 && ctx.accounts.favor_market_account.voter_id == ctx.accounts.physical_transaction.metadata.buyer {
            // ctx.accounts.favor_market_account.dispute_discounts += 1;
        }

        if let Some(auth_bump) = ctx.bumps.get("physical_auth"){
            close_dispute_helper(
                ctx.accounts.dispute_program.to_account_info(),
                ctx.accounts.phys_dispute.to_account_info(),
                ctx.accounts.funder.to_account_info(),
                ctx.accounts.physical_auth.to_account_info(),
                ctx.accounts.physical_program.to_account_info(),
                &[&[b"market_authority", &[*auth_bump]]]
            )
        }else{
            return err!(PhysicalMarketErrors::InvalidAuthBump)
        }?;

        orbit_transaction::cpi::clear_seller_physical_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearSellerPhysicalTransactions{
                    transactions_log: ctx.accounts.seller_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.physical_auth.to_account_info(),
                    caller: ctx.accounts.physical_program.to_account_info()
                }
            ),
            ctx.accounts.physical_transaction.metadata.seller_tx_index
        )?;

        orbit_transaction::cpi::clear_buyer_physical_transaction(
            CpiContext::new(
                ctx.accounts.transaction_program.to_account_info(),
                orbit_transaction::cpi::accounts::ClearBuyerPhysicalTransactions{
                    transactions_log: ctx.accounts.buyer_transactions_log.to_account_info(),
                    caller_auth: ctx.accounts.physical_auth.to_account_info(),
                    caller: ctx.accounts.physical_program.to_account_info()
                }
            ),
            ctx.accounts.physical_transaction.metadata.seller_tx_index
        )?;

        ctx.accounts.physical_transaction.metadata.transaction_state = TransactionState::Closed;
        Ok(())
    }
}

///////////////////////////////////////////////////////////////////////////////////////
/// SELLER CONFIRMATIONS

#[derive(Accounts)]
pub struct SellerConfirmationsContext<'info>{
    #[account(
        mut,
        constraint = physical_transaction.metadata.transaction_state == TransactionState::BuyerFunded
    )]
    pub physical_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        constraint = seller_market_account.voter_id == physical_transaction.metadata.seller
    )]
    pub seller_market_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut,
        seeds = [
            b"seller_transactions",
            (&(orbit_transaction::TransactionType::Physical).try_to_vec()?).as_slice(),
            &seller_market_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub seller_transactions: Box<Account<'info, SellerOpenTransactions>>,

    #[account(
        address = seller_market_account.wallet
    )]
    pub wallet: Signer<'info>,
}

pub fn update_shipping(ctx: Context<SellerConfirmationsContext>, enc_shipping: [u8; 64]) -> Result<()>{
    ctx.accounts.physical_transaction.shipping = enc_shipping;
    ctx.accounts.physical_transaction.metadata.transaction_state = TransactionState::Shipped;
    Ok(())
}


/////////////////////////////////////////////////////////////////////////////////////////////
/// BUYER CONFIRMATIONS

#[derive(Accounts)]
pub struct BuyerConfirm<'info>{
    #[account(
        mut,
        constraint = physical_transaction.metadata.transaction_state == TransactionState::Shipped
    )]
    pub physical_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        mut
    )]
    pub buyer_account: Account<'info, OrbitMarketAccount>,

    #[account(
        seeds = [
            b"buyer_transactions",
            (&(orbit_transaction::TransactionType::Physical).try_to_vec()?).as_slice(),
            &buyer_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub buyer_transactions: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        address = buyer_account.wallet
    )]
    pub buyer_wallet: Signer<'info>,
}

pub fn confirm_delivery(ctx: Context<BuyerConfirm>) -> Result<()>{
    ctx.accounts.physical_transaction.metadata.transaction_state = TransactionState::BuyerConfirmedDelivery;
    Ok(())
}

pub fn confirm_product(ctx: Context<BuyerConfirm>) -> Result<()>{
    if ctx.accounts.physical_transaction.metadata.transaction_state != TransactionState::BuyerConfirmedDelivery{
        return err!(PhysicalMarketErrors::DidNotConfirmDelivery);
    }
    ctx.accounts.physical_transaction.metadata.transaction_state = TransactionState::BuyerConfirmedProduct;
    Ok(())
}

/////////////////////////////////////////////////////////////////////////////////////////////
/// ACCOUNT HELPERS (leave a review)

#[derive(Accounts)]
pub struct LeaveReview<'info>{
    /////////////////////////////////////////////////
    /// TX
    #[account(
        mut,
        constraint = physical_transaction.metadata.transaction_state == TransactionState::Closed
    )]
    pub physical_transaction: Account<'info, PhysicalTransaction>,

    /////////////////////////////////////////////////
    /// REVIEW RELATED
    #[account(
        mut,
        constraint = 
        (reviewer.voter_id == physical_transaction.metadata.seller) ||
        (reviewer.voter_id == physical_transaction.metadata.buyer)
    )]
    pub reviewed_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        constraint = 
        (reviewer.voter_id == physical_transaction.metadata.seller) ||
        (reviewer.voter_id == physical_transaction.metadata.buyer),
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub reviewer: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        address = reviewer.wallet
    )]
    pub wallet: Signer<'info>,

    /////////////////////////////////////////////////
    /// EXTRANEOUS CPI
    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub phys_auth: SystemAccount<'info>,
    
    pub physical_program: Program<'info, OrbitPhysicalMarket>,

    pub accounts_program: Program<'info, OrbitMarketAccounts>
}

impl <'a> OrbitMarketAccountTrait<'a, LeaveReview<'a>> for PhysicalTransaction{
 
    fn leave_review(ctx: Context<LeaveReview>, rating: u8) -> Result<()>{
        if ctx.accounts.reviewer.key() == ctx.accounts.reviewed_account.key(){
            return err!(ReviewErrors::InvalidReviewAuthority)
        };
        if rating == 0 || rating > 5{
            return err!(ReviewErrors::InvalidReviewAuthority)
        };

        if ctx.accounts.physical_transaction.metadata.seller == ctx.accounts.reviewer.voter_id && !ctx.accounts.physical_transaction.metadata.reviews.seller{
            if let Some(auth_bump) = ctx.bumps.get("phys_auth"){
                orbit_transaction::submit_rating_with_signer!(
                    ctx.accounts.accounts_program.to_account_info(),
                    ctx.accounts.reviewed_account.to_account_info(),
                    ctx.accounts.phys_auth.to_account_info(),
                    ctx.accounts.physical_program.to_account_info(),
                    &[&[b"market_authority", &[*auth_bump]]],
                    rating
                )?;
                ctx.accounts.physical_transaction.metadata.reviews.seller = true;
                 
            }else{
                return err!(MarketAccountErrors::CannotCallOrbitAccountsProgram)
            };
        }else
        if ctx.accounts.physical_transaction.metadata.buyer == ctx.accounts.reviewer.voter_id  && !ctx.accounts.physical_transaction.metadata.reviews.buyer{
            if let Some(auth_bump) = ctx.bumps.get("phys_auth"){
                orbit_transaction::submit_rating_with_signer!(
                    ctx.accounts.accounts_program.to_account_info(),
                    ctx.accounts.reviewed_account.to_account_info(),
                    ctx.accounts.phys_auth.to_account_info(),
                    ctx.accounts.physical_program.to_account_info(),
                    &[&[b"market_authority", &[*auth_bump]]],
                    rating
                )?;
            }else{
                return err!(MarketAccountErrors::CannotCallOrbitAccountsProgram)
            };
        }else
        {
            return err!(ReviewErrors::InvalidReviewAuthority)
        };

        Ok(())
    }

}
