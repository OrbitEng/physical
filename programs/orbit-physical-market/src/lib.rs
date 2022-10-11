use anchor_lang::prelude::*;

pub mod structs;
pub mod accessors;
pub mod errors;

pub use structs::*;
pub use accessors::*;

declare_id!("HgZsfGTHEygTLSRDoKZkQMkJPm8jesNtUgjSQgFwVh7S");

#[program]
pub mod orbit_physical_market {
    use super::*;

    use orbit_transaction::transaction_trait::OrbitTransactionTrait;
    use orbit_dispute::OrbitDisputableTrait;
    use market_accounts::structs::OrbitMarketAccountTrait;

    //////////////////////////////////////////////
    /// TRANSACTION HANDLERS

    /// SOL
    pub fn open_transaction_sol(ctx: Context<OpenPhysicalTransactionSol>, seller_index: u8, buyer_index: u8, price: u64, use_discount: bool) -> Result<()>{
        PhysicalTransaction::open_sol(ctx, seller_index, buyer_index, price, use_discount)
    }

    pub fn close_transaction_sol<'a>(ctx: Context<'_, '_, '_, 'a, ClosePhysicalTransactionSol<'a>>) -> Result<()>{
        PhysicalTransaction::close_sol(ctx)
    }

    pub fn fund_escrow_sol(ctx: Context<FundEscrowSol>) -> Result<()>{
        PhysicalTransaction::fund_escrow_sol(ctx)
    }

    pub fn seller_early_decline_sol(ctx: Context<SellerEarlyDeclineSol>) -> Result<()>{
        PhysicalTransaction::seller_early_decline_sol(ctx)
    }

    /// SPL
    pub fn open_transaction_spl(ctx: Context<OpenPhysicalTransactionSpl>, seller_index: u8, buyer_index: u8, price: u64, use_discount: bool) -> Result<()>{
        PhysicalTransaction::open_spl(ctx, seller_index, buyer_index, price, use_discount)
    }

    pub fn close_transaction_spl<'a>(ctx: Context<'_, '_, '_, 'a, ClosePhysicalTransactionSpl<'a>>) -> Result<()>{
        PhysicalTransaction::close_spl(ctx)
    }

    pub fn fund_escrow_spl(ctx: Context<FundEscrowSpl>) -> Result<()>{
        PhysicalTransaction::fund_escrow_spl(ctx)
    }

    pub fn seller_early_decline_spl(ctx: Context<SellerEarlyDeclineSpl>) -> Result<()>{
        PhysicalTransaction::seller_early_decline_spl(ctx)
    }

    /// COMMON
    pub fn close_transaction_account(ctx: Context<CloseTransactionAccount>) -> Result<()>{
        PhysicalTransaction::close_transaction_account(ctx)
    }

    ////////////////////////////////////
    /// DISPUTE RELATED

    pub fn open_dispute(ctx: Context<OpenPhysicalDispute>, threshold: u8) -> Result<()>{
        PhysicalTransaction::open_dispute(ctx, threshold)
    }

    pub fn close_dispute_sol(ctx: Context<ClosePhysicalDisputeSol>) -> Result<()>{
        PhysicalTransaction::close_dispute_sol(ctx)
    }

    pub fn close_dispute_spl(ctx: Context<ClosePhysicalDisputeSpl>) -> Result<()>{
        PhysicalTransaction::close_dispute_spl(ctx)
    }

    /////////////////////////////////////////////////
    /// REVIEW RELATED
    
    pub fn leave_review(ctx: Context<LeaveReview>, rating: u8) -> Result<()>{
        PhysicalTransaction::leave_review(ctx, rating)
    }

}