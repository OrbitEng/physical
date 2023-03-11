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
    #[account(
        init,
        payer = buyer_wallet,
        space = 400,
        seeds = [
            b"orbit_physical_transaction",
            seller_transactions_log.key().as_ref(),
            &[seller_tx_index]
        ],
        bump
    )]
    pub physical_transaction: Box<Account<'info, PhysicalTransaction>>,
    
    #[account(
        seeds = [
            b"orbit_escrow_account",
            physical_transaction.key().as_ref(),
            buyer_transactions_log.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: SystemAccount<'info>,

    #[account(
        mut,
        constraint = phys_product.quantity > 0,
        constraint = phys_product.metadata.owner_catalog == seller_market_account.voter_id
    )] 
    pub phys_product: Box<Account<'info, PhysicalProduct>>,
    
    #[account(
        mut,
        seeds = [
            b"buyer_transactions",
            (&(orbit_transaction::TransactionType::Digital).try_to_vec()?).as_slice(),
            &buyer_market_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub buyer_transactions_log: Box<Account<'info, BuyerOpenTransactions>>,

    #[account(
        mut
    )]
    pub buyer_market_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = buyer_market_account.wallet
    )]
    pub buyer_wallet: Signer<'info>,

    #[account(    )]
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
            (&(orbit_transaction::TransactionType::Commissions).try_to_vec()?).as_slice(),
            &seller_market_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
    )]
    pub seller_transactions_log: Box<Account<'info, SellerOpenTransactions>>,

    
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
    pub escrow_account: SystemAccount<'info>,

    ///////////////////////////////////////////////////
    /// BUYER SELLER ACCOUNTS
    
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
            (&(orbit_transaction::TransactionType::Commissions).try_to_vec()?).as_slice(),
            &buyer_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
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
        constraint = seller_account.voter_id == physical_transaction.metadata.seller
    )]
    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        seeds = [
            b"seller_transactions",
            (&(orbit_transaction::TransactionType::Commissions).try_to_vec()?).as_slice(),
            &seller_account.voter_id.to_le_bytes()
        ], 
        bump,
        seeds::program = &orbit_transaction::id()
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

    pub product_program: Program<'info, OrbitProduct>,

    pub transaction_program: Program<'info, OrbitTransaction>
}

#[derive(Accounts)]
pub struct FundEscrowSol<'info>{
    ////////////////////////////////////////////
    /// TX
    #[account(
        mut,
        constraint = physical_transaction.metadata.transaction_state == TransactionState::SellerConfirmed
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
    pub escrow_account: SystemAccount<'info>,

    ////////////////////////////////////////////
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
        constraint = buyer_market_account.voter_id == physical_transaction.metadata.buyer
    )]
    pub buyer_market_account: Box<Account<'info, OrbitMarketAccount>>,
    

    #[account(
        mut,
        address = buyer_market_account.wallet
    )]
    pub buyer_wallet: Signer<'info>
}

#[derive(Accounts)]
pub struct SellerEarlyDeclineSol<'info>{
    ////////////////////////////////////////////
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
    pub escrow_account: SystemAccount<'info>,

    ///////////////////////////////////////////////////
    /// BUYER SELLER ACCOUNTS
    
    /// BUYER
    #[account(
        mut,
        constraint = buyer_account.voter_id == physical_transaction.metadata.buyer
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,
    
    /// BUYER
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
        address = buyer_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>,
    
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
    pub escrow_account: SystemAccount<'info>,
    
    ////////////////////////////////////////////
    /// DISPUTE
    #[account(
        mut,
        constraint = phys_dispute.dispute_transaction == physical_transaction.key(),
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
        address = buyer_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>,

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