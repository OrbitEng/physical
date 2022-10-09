use anchor_lang::prelude::*;
use orbit_product::product_struct::OrbitProduct;

pub mod structs;
pub mod accessors;
pub mod errors;

pub use structs::*;
pub use accessors::*;

declare_id!("97yvrxpWXrsurDgiWiskJ4KcQhFZJF6SrLoUYA53bpBL");

#[program]
pub mod orbit_physical_market {
    use super::*;

    use orbit_product::product_trait::OrbitProductTrait;
    use orbit_transaction::transaction_trait::OrbitTransactionTrait;
    use orbit_dispute::OrbitDisputableTrait;
    use market_accounts::structs::OrbitMarketAccountTrait;

    ///////////////////////////////////////////////////
    /// INITIALIZERS
    
    pub fn init_recent_catalog(ctx: Context<CreatePhysRecentCatalog>) -> Result<()>{
        recent_phys_catalog_handler(ctx)
    }

    //////////////////////////////////////////////
    /// TRANSACTION HANDLERS

    /// SOL
    pub fn open_transaction_sol(ctx: Context<OpenPhysicalTransactionSol>, price: u64, use_discount: bool) -> Result<()>{
        Physicalorbit_transaction::open_sol(ctx, price, use_discount)
    }

    pub fn close_transaction_sol<'a>(ctx: Context<'_, '_, '_, 'a, ClosePhysicalTransactionSol<'a>>) -> Result<()>{
        Physicalorbit_transaction::close_sol(ctx)
    }

    pub fn fund_escrow_sol(ctx: Context<FundEscrowSol>) -> Result<()>{
        Physicalorbit_transaction::fund_escrow_sol(ctx)
    }

    /// SPL
    pub fn open_transaction_spl(ctx: Context<OpenPhysicalTransactionSpl>, price: u64, use_discount: bool) -> Result<()>{
        Physicalorbit_transaction::open_spl(ctx, price, use_discount)
    }

    pub fn close_transaction_spl<'a>(ctx: Context<'_, '_, '_, 'a, ClosePhysicalTransactionSpl<'a>>) -> Result<()>{
        Physicalorbit_transaction::close_spl(ctx)
    }

    pub fn fund_escrow_spl(ctx: Context<FundEscrowSpl>) -> Result<()>{
        Physicalorbit_transaction::fund_escrow_spl(ctx)
    }

    pub fn close_transaction_account(ctx: Context<CloseTransactionAccount>) -> Result<()>{
        Physicalorbit_transaction::close_transaction_account(ctx)
    }

    ////////////////////////////////////
    /// DISPUTE RELATED

    pub fn open_dispute(ctx: Context<OpenPhysicalDispute>, threshold: u8) -> Result<()>{
        Physicalorbit_transaction::open_dispute(ctx, threshold)
    }

    pub fn close_dispute_sol(ctx: Context<ClosePhysicalDisputeSol>) -> Result<()>{
        Physicalorbit_transaction::close_dispute_sol(ctx)
    }

    pub fn close_dispute_spl(ctx: Context<ClosePhysicalDisputeSpl>) -> Result<()>{
        Physicalorbit_transaction::close_dispute_spl(ctx)
    }

    /////////////////////////////////////////////////
    /// REVIEW RELATED
    
    pub fn leave_review(ctx: Context<LeaveReview>, rating: u8) -> Result<()>{
        Physicalorbit_transaction::leave_review(ctx, rating)
    }

}