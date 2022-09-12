use anchor_lang::prelude::*;
use market_accounts::{
    OrbitMarketAccount,
    program::OrbitMarketAccounts
};
use dispute::{
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
        constraint = phys_product.metadata.currency != System::id()
    )]
    pub phys_product: Account<'info, PhysicalProduct>,

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

    #[account(mut)]
    pub buyer_account: Account<'info, OrbitMarketAccount>,

    #[account(mut)]
    pub buyer_wallet: Signer<'info>,

    #[account(
        address = buyer_account.master_pubkey
    )]
    pub buyer_auth: Signer<'info>,

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
        address = phys_transaction.metadata.buyer
    )]
    pub buyer_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>,

    #[account(
        address = phys_transaction.metadata.seller
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
        constraint = (authority.key() == seller_account.master_pubkey) || (authority.key() == buyer_account.master_pubkey)
    )] // enforce constraints. only certain people should be able to close
    pub authority: Signer<'info>,

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
    pub multisig_address: Account<'info, Multisig>,

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
        address = phys_transaction.metadata.buyer
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
        address = buyer_account.master_pubkey
    )]
    pub buyer_auth: Signer<'info>,

    #[account(
        address = buyer_spl_wallet.owner
    )]
    pub wallet_owner: Signer<'info>,

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
        constraint = (phys_transaction.metadata.seller == favor_market_account.key()) || (phys_transaction.metadata.buyer == favor_market_account.key()),
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
        constraint = favor_token_account.owner == favor_market_account.wallet
    )]
    pub favor_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = phys_dispute.favor
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
    pub multisig_address: Account<'info, Multisig>,

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
