use anchor_lang::prelude::*;
use market_accounts::{
    OrbitMarketAccount,
    program::OrbitMarketAccounts
};
use orbit_product::{ListingsStruct, program::OrbitProduct};
use orbit_dispute::{
    program::Dispute,
    DisputeState,
    OrbitDispute
};
use orbit_transaction::{transaction_struct::TransactionState, SellerOpenTransactions, BuyerOpenTransactions, program::OrbitTransaction};
use orbit_product::PhysicalProduct;
use crate::{
    PhysicalTransaction,
    program::OrbitPhysicalMarket
};

/////////////////////////////////
/// BASE TX UTILS

#[derive(Accounts)]
#[instruction(seller_tx_index: u8)]
pub struct OpenPhysicalTransactionSol<'info>{
    ////////////////////////////////
    /// TX
    #[account(
        init,
        payer = buyer_wallet,
        space = 400,
        seeds = [
            b"orbit_physical_transaction",
            [seller_tx_index].as_ref()
        ],
        bump
    )]
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,
    
    #[account(
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: SystemAccount<'info>,

    #[account(
        mut,
        constraint = phys_product.metadata.currency == System::id(),
        constraint = phys_product.quantity > 0
    )] 
    pub phys_product: Box<Account<'info, PhysicalProduct>>,
    
    //////////////////////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(mut)]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        constraint = buyer_market_account.wallet == buyer_wallet.key(),
        constraint = buyer_market_account.buyer_physical_transactions == buyer_transactions_log.key()
    )]
    pub buyer_market_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = buyer_transactions_log.buyer_wallet
    )]
    pub buyer_wallet: Signer<'info>,
    
    /// SELLER
    #[account(
        address = phys_product.metadata.owner_catalog
    )]
    pub seller_listings: Box<Account<'info, ListingsStruct>>,

    #[account(
        mut,
        constraint = seller_transactions_log.seller_wallet == seller_listings.listings_owner
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    /////////////////////////////////
    /// EXTRANEOUS
    
    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,
    
    pub physical_program: Program<'info, OrbitPhysicalMarket>,

    pub transaction_program: Program<'info, OrbitTransaction>,

    pub market_account_program: Program<'info, OrbitMarketAccounts>,
    
    pub product_program: Program<'info, OrbitProduct>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClosePhysicalTransactionSol<'info>{
    ////////////////////////////////////////////
    /// TX
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::BuyerConfirmedProduct,
    )]
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,

    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref()
        ],
        bump,
        address = phys_transaction.metadata.escrow_account
    )]
    pub escrow_account: SystemAccount<'info>,

    ///////////////////////////////////////////////////
    /// BUYER SELLER ACCOUNTS
    
    /// BUYER
    #[account(
        mut,
        constraint = buyer_account.buyer_physical_transactions == buyer_transactions_log.key()
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,
    
    #[account(
        mut,
        address = phys_transaction.metadata.buyer,
        has_one = buyer_wallet
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>,
    
    /// SELLER
    #[account(
        mut,
        constraint = seller_account.seller_physical_transactions == seller_transactions_log.key()
    )]
    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = phys_transaction.metadata.seller,
        has_one = seller_wallet
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    #[account(
        mut,
        address = seller_account.wallet
    )]
    pub seller_wallet: SystemAccount<'info>,

    //////////////////////////////////
    /// CPI AND EXTRANEOUS
    
    #[account(
        mut,
        address = Pubkey::new(orbit_addresses::MULTISIG_SIGNER)
    )]
    pub multisig_wallet: SystemAccount<'info>,

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,

    pub physical_program: Program<'info, OrbitPhysicalMarket>,
    
    pub market_account_program: Program<'info, OrbitMarketAccounts>,

    pub transaction_program: Program<'info, OrbitTransaction>
}

#[derive(Accounts)]
pub struct FundEscrowSol<'info>{
    ////////////////////////////////////////////
    /// TX
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::SellerConfirmed,
        constraint = phys_transaction.metadata.escrow_account == escrow_account.key()
    )]
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,
    
    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: SystemAccount<'info>,

    ////////////////////////////////////////////
    /// BUYER SELLER

    /// BUYER
    #[account(
        mut,
        address = phys_transaction.metadata.buyer,
        has_one = buyer_wallet
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(mut)]
    pub buyer_wallet: Signer<'info>
}

#[derive(Accounts)]
pub struct SellerEarlyDeclineSol<'info>{
    ////////////////////////////////////////////
    /// TX
    #[account(
        mut
    )]
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,

    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref()
        ],
        bump,
        address = phys_transaction.metadata.escrow_account
    )]
    pub escrow_account: SystemAccount<'info>,

    ///////////////////////////////////////////////////
    /// BUYER SELLER ACCOUNTS
    
    /// BUYER
    #[account(
        mut,
        constraint = buyer_account.buyer_physical_transactions == buyer_transactions_log.key()
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,
    
    #[account(
        mut,
        address = phys_transaction.metadata.buyer,
        has_one = buyer_wallet
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>,
    
    /// SELLER

    #[account(
        mut,
        address = phys_transaction.metadata.seller,
        has_one = seller_wallet
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    #[account(mut)]
    pub seller_wallet: Signer<'info>,

    //////////////////////////////////
    /// CPI AND EXTRANEOUS
    
    #[account(
        mut,
        address = Pubkey::new(orbit_addresses::MULTISIG_SIGNER)
    )]
    pub multisig_wallet: SystemAccount<'info>,

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,

    pub physical_program: Program<'info, OrbitPhysicalMarket>,
    
    pub market_account_program: Program<'info, OrbitMarketAccounts>,

    pub transaction_program: Program<'info, OrbitTransaction>
}

////////////////////////////////////////////////////
/// DISPUTE UTILS

#[derive(Accounts)]
pub struct ClosePhysicalDisputeSol<'info>{
    ////////////////////////////////////////////
    /// TX
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::Frozen
    )]
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,
    
    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref()
        ],
        bump,

        address = phys_transaction.metadata.escrow_account
    )]
    pub escrow_account: SystemAccount<'info>,
    
    ////////////////////////////////////////////
    /// DISPUTE
    #[account(
        mut,
        constraint = phys_dispute.dispute_transaction == phys_transaction.key(),
        constraint = phys_dispute.dispute_state == DisputeState::Resolved,
        has_one = funder
    )]
    pub phys_dispute: Box<Account<'info, OrbitDispute>>,

    #[account(
        mut,
        seeds = [
            b"orbit_account",
            favor_wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID,
        
        constraint = favor_market_account.voter_id == phys_dispute.favor
    )]
    pub favor_market_account: Box<Account<'info, OrbitMarketAccount>>,
        
    #[account(
        mut,
        address = favor_market_account.wallet
    )]
    pub favor_wallet: SystemAccount<'info>,

    #[account(mut)]
    pub funder: SystemAccount<'info>,

    ///////////////////////////////////////////////////
    /// BUYER SELLER ACCOUNTS
    
    /// BUYER
    #[account(
        mut,
        seeds = [
            b"orbit_account",
            buyer_wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID,
        constraint = buyer_account.buyer_physical_transactions == buyer_transactions_log.key()
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,
    
    #[account(
        mut,
        address = phys_transaction.metadata.buyer,
        has_one = buyer_wallet
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,
    
    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>,

    /// SELLER
    #[account(
        mut,
        constraint = seller_account.seller_physical_transactions == seller_transactions_log.key()
    )]
    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = phys_transaction.metadata.seller,
        constraint = seller_transactions_log.seller_wallet == seller_account.wallet
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    //////////////////////////////////
    /// CPI AND EXTRANEOUS
    
    #[account(
        mut,
        address = Pubkey::new(orbit_addresses::MULTISIG_SIGNER)
    )]
    pub multisig_wallet: SystemAccount<'info>,
    
    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,
    
    pub physical_program: Program<'info, OrbitPhysicalMarket>,
    
    pub market_accounts_program: Program<'info, OrbitMarketAccounts>,
    
    pub transaction_program: Program<'info, OrbitTransaction>,

    pub dispute_program: Program<'info, Dispute>,

}