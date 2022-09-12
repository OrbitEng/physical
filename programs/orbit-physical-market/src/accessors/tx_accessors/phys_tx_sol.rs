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
pub struct OpenPhysicalTransactionSol<'info>{
    #[account(
        init,
        payer = buyer_wallet,
        space = 1000
    )]
    pub phys_transaction: Box<Account<'info, PhysicalTransaction>>,

    #[account(
        constraint = phys_product.metadata.currency == System::id()
    )]
    pub phys_product: Account<'info, PhysicalProduct>,

    #[account(
        seeds = [
            b"orbit_escrow_account",
            phys_transaction.key().as_ref()
        ],
        bump
    )]
    pub escrow_account: SystemAccount<'info>,

    #[account(mut)]
    pub buyer_account: Account<'info, OrbitMarketAccount>,

    #[account(mut)]
    pub buyer_wallet: Signer<'info>,

    #[account(
        address = buyer_account.master_pubkey
    )]
    pub buyer_auth: Signer<'info>,

    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct ClosePhysicalTransactionSol<'info>{
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::BuyerConfirmedProduct,
    )]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        address = phys_transaction.metadata.buyer
    )]
    pub buyer_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut,
        address = buyer_account.wallet
    )]
    pub buyer_wallet: SystemAccount<'info>,

    #[account(
        address = phys_transaction.metadata.seller
    )]
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
        bump,

        address = phys_transaction.metadata.escrow_account
    )]
    pub escrow_account: SystemAccount<'info>,

    #[account(
        constraint = (authority.key() == seller_account.master_pubkey) || (authority.key() == buyer_account.master_pubkey)
    )]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,

    #[account(
        address = market_accounts::ID
    )]
    pub market_account_program: Program<'info, OrbitMarketAccounts>,

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
        bump = multisig_address.nonce
    )]
    pub multisig_wallet: SystemAccount<'info>,
}

#[derive(Accounts)]
pub struct FundEscrowSol<'info>{
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

    #[account(mut)]
    pub buyer_wallet: Signer<'info>,

    #[account(
        address = buyer_account.master_pubkey
    )]
    pub buyer_auth: Signer<'info>
}

////////////////////////////////////////////////////
/// DISPUTE UTILS

#[derive(Accounts)]
pub struct ClosePhysicalDisputeSol<'info>{
    #[account(
        mut,
        constraint = phys_transaction.metadata.transaction_state == TransactionState::Frozen,
        constraint = (phys_transaction.metadata.seller == favor_market_account.key()) || (phys_transaction.metadata.buyer == favor_market_account.key()),
    )]
    pub phys_transaction: Account<'info, PhysicalTransaction>,

    #[account(
        mut,
        constraint = phys_dispute.dispute_transaction == phys_transaction.key(),
        constraint = phys_dispute.dispute_state == DisputeState::Resolved,
        has_one = funder
    )]
    pub phys_dispute: Account<'info, OrbitDispute>,

    #[account(
        mut,
        address = favor_market_account.wallet
    )]
    pub favor: SystemAccount<'info>,

    #[account(
        mut,
        address = phys_dispute.favor
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
        bump,

        address = phys_transaction.metadata.escrow_account
    )]
    pub escrow_account: SystemAccount<'info>,

    #[account(
        seeds = [b"market_authority"],
        bump
    )]
    pub physical_auth: SystemAccount<'info>,

    pub physical_program: Program<'info, OrbitPhysicalMarket>,

    pub dispute_program: Program<'info, Dispute>,

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
        bump = multisig_address.nonce
    )]
    pub multisig_wallet: SystemAccount<'info>,
    
    #[account(
        address = phys_transaction.metadata.buyer
    )]
    pub buyer_account: Account<'info, OrbitMarketAccount>,

    #[account(mut)]
    pub buyer_wallet: Signer<'info>,
}
