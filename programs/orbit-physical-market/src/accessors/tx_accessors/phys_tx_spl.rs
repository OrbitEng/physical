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
use anchor_spl::token::{
    TokenAccount,
    Mint,
    Token
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
pub struct OpenPhysicalTransactionSpl<'info>{
    //////////////////////////////////
    /// TX
    #[account(
        init,
        payer = buyer_wallet,
        space = 1000,
        seeds = [
            b"orbit_physical_transaction",
            seller_transactions_log.key().as_ref(),
            [seller_tx_index].as_ref()
        ],
        bump
    )]
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,

    #[account(
        init,
        token::mint = token_mint,
        token::authority = physical_auth,
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump,
        payer = buyer_wallet
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    //////////////////////////////////
    /// PRODUCT
    #[account(
        address = phys_product.metadata.currency
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = phys_product.metadata.currency != System::id(),
        constraint = phys_product.quantity > 0
    )]
    pub phys_product: Box<Account<'info, PhysicalProduct>>,
    
    //////////////////////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(
        mut,
        has_one = buyer_wallet
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        constraint = buyer_market_account.wallet == buyer_wallet.key(),
        constraint = buyer_market_account.buyer_physical_transactions == buyer_transactions_log.key()
    )]
    pub buyer_market_account: Box<Account<'info, OrbitMarketAccount>>,
    
    #[account(mut)]
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

    //////////////////////////////////////////////////
    /// EXTRANEOUS CPI
    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,

    pub physical_program: Program<'info, OrbitPhysicalMarket>,

    pub market_account_program: Program<'info, OrbitMarketAccounts>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    pub product_program: Program<'info, OrbitProduct>,
    
    pub transaction_program: Program<'info, OrbitTransaction>,

    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct ClosePhysicalTransactionSpl<'info>{
    //////////////////////////////////
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
            phys_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump,
        
        address = phys_transaction.metadata.escrow_account
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    //////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(
        mut,
        constraint = buyer_account.wallet == buyer_transactions_log.buyer_wallet
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = phys_transaction.metadata.buyer
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        token::authority = buyer_transactions_log.buyer_wallet
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    /// SELLER
    #[account(
        mut,
        constraint = seller_account.wallet == seller_transactions_log.seller_wallet
    )]
    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = phys_transaction.metadata.seller
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    #[account(
        mut,
        token::authority = seller_transactions_log.seller_wallet
    )]
    pub seller_token_account: Account<'info, TokenAccount>,

    
    //////////////////////////////////
    /// CPI AND EXTRANEOUS
    
    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,
    
    #[account(
        token::authority = Pubkey::new(orbit_addresses::MULTISIG_SIGNER)
    )]
    pub multisig_ata: Account<'info, TokenAccount>,

    pub market_account_program: Program<'info, OrbitMarketAccounts>,
    
    pub physical_program: Program<'info, OrbitPhysicalMarket>,

    pub transaction_program: Program<'info, OrbitTransaction>,

    pub token_program: Program<'info, Token>,
    
}

#[derive(Accounts)]
pub struct SellerEarlyDeclineSpl<'info>{
    //////////////////////////////////
    /// TX
    #[account(
        mut
    )]
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,
    
    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump,
        
        address = phys_transaction.metadata.escrow_account
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    //////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(
        mut,
        constraint = buyer_account.wallet == buyer_transactions_log.buyer_wallet
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = phys_transaction.metadata.buyer
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        token::authority = buyer_transactions_log.buyer_wallet
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    /// SELLER

    #[account(
        mut,
        address = phys_transaction.metadata.seller,
        has_one = seller_wallet
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    pub seller_wallet: Signer<'info>,

    
    //////////////////////////////////
    /// CPI AND EXTRANEOUS
    
    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,

    pub market_account_program: Program<'info, OrbitMarketAccounts>,
    
    pub physical_program: Program<'info, OrbitPhysicalMarket>,

    pub transaction_program: Program<'info, OrbitTransaction>,

    pub token_program: Program<'info, Token>,
    
}

#[derive(Accounts)]
pub struct FundEscrowSpl<'info>{
    ////////////////////////////////////////////
    /// TX
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::SellerConfirmed,
    )]
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,
    
    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump,
        address = phys_transaction.metadata.escrow_account
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    ////////////////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(
        mut,
        address = phys_transaction.metadata.buyer
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        token::authority = buyer_wallet.key()
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    #[account(
        address = buyer_transactions_log.buyer_wallet
    )]
    pub buyer_wallet: Signer<'info>,

    //////////////////////////////////
    /// CPI AND EXTRANEOUS

    pub token_program: Program<'info, Token>
}

////////////////////////////////////////////////////
/// DISPUTE UTILS


#[derive(Accounts)]
pub struct ClosePhysicalDisputeSpl<'info>{
    ////////////////////////////////////////////
    /// TX
    
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::Frozen,
        constraint = phys_transaction.metadata.escrow_account == escrow_account.key()
    )]
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,
    
    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: Account<'info, TokenAccount>,
    
    /////////////////////////////////////////////////////
    /// DISPUTE RELATED
    
    #[account(
        mut,
        constraint = phys_dispute.dispute_transaction == phys_transaction.key(),
        constraint = phys_dispute.dispute_state == DisputeState::Resolved,
        has_one = funder
    )]
    pub phys_dispute: Box<Account<'info, OrbitDispute>>,
    
    #[account(
        mut,
        token::authority = favor_market_account.wallet
    )]
    pub favor_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = favor_market_account.voter_id == phys_dispute.favor
    )]
    pub favor_market_account: Box<Account<'info, OrbitMarketAccount>>,
    
    #[account(mut)]
    pub funder: SystemAccount<'info>,

    
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
        constraint = buyer_transactions_log.buyer_wallet == buyer_account.wallet
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        token::authority = buyer_transactions_log.buyer_wallet
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    
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
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,

    #[account(
        token::authority = Pubkey::new(orbit_addresses::MULTISIG_SIGNER)
    )]
    pub multisig_ata: Account<'info, TokenAccount>,

    pub physical_program: Program<'info, OrbitPhysicalMarket>,

    pub dispute_program: Program<'info, Dispute>,
    
    pub transaction_program: Program<'info, OrbitTransaction>,

    pub token_program: Program<'info, Token>,

    pub market_accounts_program: Program<'info, OrbitMarketAccounts>,
}
