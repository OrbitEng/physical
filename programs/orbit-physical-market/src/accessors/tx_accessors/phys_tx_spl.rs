use anchor_lang::prelude::*;
use market_accounts::{
    OrbitMarketAccount,
    program::OrbitMarketAccounts
};
use orbit_product::{program::OrbitProduct, ListingsStruct};
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
    pub physical_transaction: Box<Account<'info, PhysicalTransaction>>,

    #[account(
        init,
        token::mint = token_mint,
        token::authority = physical_auth,
        seeds = [
            b"orbit_escrow_account",
            physical_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump,
        payer = buyer_wallet
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    //////////////////////////////////
    /// PRODUCT
    #[account()]
    pub token_mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = phys_product.quantity > 0,
        constraint = phys_product.metadata.owner_catalog == seller_market_account.voter_id
    )]
    pub phys_product: Box<Account<'info, PhysicalProduct>>,
    
    //////////////////////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(
        mut,
        seeds = [
            b"buyer_transactions",
            (&(orbit_transaction::TransactionType::Physical).try_to_vec()?).as_slice(),
            &buyer_market_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        constraint = buyer_market_account.wallet == buyer_wallet.key()
    )]
    pub buyer_market_account: Box<Account<'info, OrbitMarketAccount>>,
    
    #[account(mut)]
    pub buyer_wallet: Signer<'info>,
    
    /// SELLER
    pub seller_market_account: Account<'info, OrbitMarketAccount>,
    
    #[account(
        mut,
        seeds = [
            b"vendor_listings",
            (&(orbit_product::ListingsType::Commissions).try_to_vec()?).as_slice(),
            &seller_market_account.voter_id.to_le_bytes()
        ],
        bump,
        seeds::program = &orbit_product::id()
    )]
    pub seller_listings: Account<'info, ListingsStruct>,

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
        constraint = physical_transaction.metadata.transaction_state == TransactionState::BuyerConfirmedProduct,
    )]
    pub physical_transaction: Box<Account<'info, PhysicalTransaction>>,

    #[account(
        mut,
        constraint = phys_product.metadata.index == physical_transaction.metadata.product,
        constraint = phys_product.metadata.owner_catalog == seller_account.voter_id
    )] 
    pub phys_product: Box<Account<'info, PhysicalProduct>>,
    
    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            physical_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    //////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(
        mut,
        constraint = buyer_account.voter_id == physical_transaction.metadata.buyer
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        seeds = [
            b"buyer_transactions",
            (&(orbit_transaction::TransactionType::Physical).try_to_vec()?).as_slice(),
            &buyer_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        token::authority = buyer_account.wallet
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    /// SELLER
    #[account(
        mut,
        constraint = seller_account.voter_id == physical_transaction.metadata.seller
    )]
    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        seeds = [
            b"seller_transactions",
            (&(orbit_transaction::TransactionType::Physical).try_to_vec()?).as_slice(),
            &seller_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    #[account(
        mut,
        token::authority = seller_account.wallet
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
    
    pub product_program: Program<'info, OrbitProduct>,

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
    pub physical_transaction: Box<Account<'info, PhysicalTransaction>>,
    
    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            physical_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    //////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(
        constraint = buyer_market_account.voter_id == physical_transaction.metadata.buyer
    )]
    pub buyer_market_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut,
        seeds = [
            b"buyer_transactions",
            (&(orbit_transaction::TransactionType::Physical).try_to_vec()?).as_slice(),
            &buyer_market_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        token::authority = buyer_market_account.wallet
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    /// SELLER

    #[account(
        mut,
        constraint = seller_account.voter_id == physical_transaction.metadata.seller
    )]
    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,
    
    #[account(
        mut,
        seeds = [
            b"seller_transactions",
            (&(orbit_transaction::TransactionType::Physical).try_to_vec()?).as_slice(),
            &seller_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    #[account(
        address = seller_account.wallet
    )]
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
        constraint = physical_transaction.metadata.transaction_state == TransactionState::SellerConfirmed,
    )]
    pub physical_transaction: Box<Account<'info, PhysicalTransaction>>,
    
    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            physical_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    ////////////////////////////////////////////
    /// BUYER SELLER
    
    /// BUYER
    #[account(
        constraint = buyer_market_account.voter_id == physical_transaction.metadata.buyer
    )]
    pub buyer_market_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut,
        seeds = [
            b"buyer_transactions",
            (&(orbit_transaction::TransactionType::Physical).try_to_vec()?).as_slice(),
            &buyer_market_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        token::authority = buyer_wallet.key()
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    #[account(
        address = buyer_market_account.wallet
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
        constraint = physical_transaction.metadata.transaction_state == TransactionState::Frozen
    )]
    pub physical_transaction: Box<Account<'info, PhysicalTransaction>>,
    
    #[account(
        mut,
        seeds = [
            b"orbit_escrow_account",
            physical_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: Account<'info, TokenAccount>,
    
    /////////////////////////////////////////////////////
    /// DISPUTE RELATED
    
    #[account(
        mut,
        constraint = phys_dispute.dispute_transaction == physical_transaction.key(),
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
        constraint = buyer_market_account.voter_id == physical_transaction.metadata.buyer
    )]
    pub buyer_market_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        seeds = [
            b"buyer_transactions",
            (&(orbit_transaction::TransactionType::Physical).try_to_vec()?).as_slice(),
            &buyer_market_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut,
        token::authority = buyer_market_account.wallet
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    
    /// SELLER
    #[account(
        mut,
        constraint = seller_account.voter_id == physical_transaction.metadata.seller
    )]
    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,
    
    #[account(
        mut,
        seeds = [
            b"seller_transactions",
            (&(orbit_transaction::TransactionType::Physical).try_to_vec()?).as_slice(),
            &seller_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
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
