use anchor_lang::prelude::*;
use market_accounts::{
    OrbitMarketAccount,
    program::OrbitMarketAccounts
};
use orbit_catalog::OrbitVendorCatalog;
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
use orbit_multisig::Multisig;
use transaction::transaction_struct::TransactionState;
use crate::{
    PhysicalTransaction,
    PhysicalProduct,
    program::OrbitPhysicalMarket
};

/////////////////////////////////
/// BASE TX UTILS

#[derive(Accounts)]
pub struct OpenPhysicalTransactionSpl<'info>{
    #[account(
        init,
        payer = buyer_wallet,
        space = 1000
    )]
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,

    #[account(
        mut,
        constraint = phys_product.metadata.currency != System::id(),
        constraint = phys_product.quantity > 0
    )]
    pub phys_product:Box<Account<'info, PhysicalProduct>>,

    #[account(
        constraint = seller_account.wallet == seller_catalog.catalog_owner
    )]
    pub seller_account:Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        address = phys_product.metadata.owner_catalog
    )]
    pub seller_catalog:Box<Account<'info, OrbitVendorCatalog>>,

    #[account(
        address = phys_product.metadata.currency
    )]
    pub token_mint: Account<'info, Mint>,

    #[account(
        init,
        token::mint = token_mint,
        token::authority = phys_auth,
        seeds = [
            b"physical_escrow_spl",
            phys_transaction.key().as_ref(),
        ],
        bump,
        payer = buyer_wallet
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            b"orbit_account",
            buyer_wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub buyer_account:Box<Account<'info, OrbitMarketAccount>>,

    #[account(mut)]
    pub buyer_wallet: Signer<'info>,

    pub system_program: Program<'info, System>,

    pub token_program: Program<'info, Token>,

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub phys_auth: SystemAccount<'info>,

    pub rent: Sysvar<'info, Rent>
}

#[derive(Accounts)]
pub struct ClosePhysicalTransactionSpl<'info>{
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::BuyerConfirmedProduct,
    )]
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,

    #[account(
        constraint = buyer_account.voter_id == phys_transaction.metadata.buyer
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        constraint = buyer_token_account.owner == buyer_account.wallet
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,

    #[account(
        constraint = seller_account.voter_id == phys_transaction.metadata.seller
    )]
    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        constraint = seller_token_account.owner == seller_account.wallet
    )]
    pub seller_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            b"physical_escrow_spl",
            phys_transaction.key().as_ref(),
        ],
        bump,

        address = phys_transaction.metadata.escrow_account
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,

    pub market_account_program: Program<'info, OrbitMarketAccounts>,

    pub token_program: Program<'info, Token>,
    
    pub physical_program: Program<'info, OrbitPhysicalMarket>,

    #[account(
        mut,
        address = Pubkey::new(orbit_addresses::MULTISIG_WALLET_ADDRESS)
    )]
    pub multisig_address: Box<Account<'info, Multisig>>,

    #[account(
        mut,
        seeds = [
            multisig_address.key().as_ref()
        ],
        bump = multisig_address.nonce,
        seeds::program = orbit_multisig::ID
    )]
    pub multisig_owner: SystemAccount<'info>,

    #[account(
        constraint = multisig_ata.owner == multisig_owner.key()
    )]
    pub multisig_ata: Account<'info, TokenAccount>,
}

#[derive(Accounts)]
pub struct FundEscrowSpl<'info>{
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::SellerConfirmed,
    )]
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,

    #[account(
        constraint = buyer_account.voter_id == phys_transaction.metadata.buyer,
        seeds = [
            b"orbit_account",
            buyer_wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        seeds = [
            b"physical_escrow_spl",
            phys_transaction.key().as_ref()
        ],
        bump,

        address = phys_transaction.metadata.escrow_account
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    #[account(
        address = buyer_account.wallet
    )]
    pub buyer_wallet: Signer<'info>,

    #[account(
        constraint = buyer_spl_wallet.owner == buyer_wallet.key()
    )]
    pub buyer_spl_wallet: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>
}

////////////////////////////////////////////////////
/// DISPUTE UTILS


#[derive(Accounts)]
pub struct ClosePhysicalDisputeSpl<'info>{
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::Frozen,
        constraint = (phys_transaction.metadata.seller == favor_market_account.voter_id) || (phys_transaction.metadata.buyer == favor_market_account.voter_id),
        constraint = phys_transaction.metadata.escrow_account == escrow_account.key()
    )]
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,

    #[account(
        mut,
        constraint = phys_dispute.dispute_transaction == phys_transaction.key(),
        constraint = phys_dispute.dispute_state == DisputeState::Resolved,
        // check this shit
        has_one = funder
    )]
    pub phys_dispute: Box<Account<'info, OrbitDispute>>,

    // wallet has to own this :P
    #[account(
        mut,
        constraint = favor_token_account.owner == favor_market_account.wallet
    )]
    pub favor_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = favor_market_account.voter_id == phys_dispute.favor
    )]
    pub favor_market_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(mut)]
    pub funder: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [
            b"physical_escrow_spl",
            phys_transaction.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,

    pub dispute_program: Program<'info, Dispute>,

    pub physical_program: Program<'info, OrbitPhysicalMarket>,

    pub token_program: Program<'info, Token>,
    
    #[account(
        mut,
        address = Pubkey::new(orbit_addresses::MULTISIG_WALLET_ADDRESS)
    )]
    pub multisig_address: Box<Account<'info, Multisig>>,

    #[account(
        mut,
        seeds = [
            multisig_address.key().as_ref()
        ],
        bump = multisig_address.nonce,
        seeds::program = orbit_multisig::ID
    )]
    pub multisig_owner: SystemAccount<'info>,

    #[account(
        constraint = multisig_ata.owner == multisig_owner.key()
    )]
    pub multisig_ata: Account<'info, TokenAccount>,
}
