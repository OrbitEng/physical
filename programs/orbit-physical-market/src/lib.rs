use anchor_lang::prelude::*;
use product::product_struct::OrbitProduct;

pub mod structs;
pub mod accessors;
pub mod errors;

pub use structs::*;
pub use accessors::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod orbit_physical_market {
    use super::*;

    use product::product_trait::OrbitProductTrait;
    use transaction::transaction_trait::OrbitTransactionTrait;
    use dispute::OrbitDisputableTrait;
    use market_accounts::structs::OrbitMarketAccountTrait;
    
    ///////////////////////////////////////////////////
    /// PRODUCT HANDLERS

    pub fn list_product(ctx: Context<ListPhysicalProduct>, prod: OrbitProduct) -> Result<()> {
        PhysicalProduct::list(ctx, prod)
    }

    pub fn unlist_product(ctx: Context<UnlistPhysicalProduct>) -> Result<()> {
        PhysicalProduct::unlist(ctx)
    }

    pub fn update_product_price(ctx: Context<UpdateProductField>, price: u64) -> Result<()>{
        update_price_handler(ctx, price)
    }

    pub fn update_product_quantity(ctx: Context<UpdateProductField>, qnt: u32) -> Result<()>{
        update_quantity_handler(ctx, qnt)
    }

    //////////////////////////////////////////////
    /// TRANSACTION HANDLERS

    pub fn open_transaction(ctx: Context<OpenPhysicalTransaction>, price: u64) -> Result<()>{
        PhysicalTransaction::open(ctx, price)
    }

    pub fn close_transaction(ctx: Context<ClosePhysicalTransaction>) -> Result<()>{
        PhysicalTransaction::close(ctx)
    }

    pub fn fund_escrow(ctx: Context<FundEscrow>) -> Result<()>{
        PhysicalTransaction::fund_escrow(ctx)
    }
    pub fn close_transaction_account(ctx: Context<CloseTransactionAccount>) -> Result<()>{
        PhysicalTransaction::close_transaction_account(ctx)
    }

    pub fn open_dispute(ctx: Context<OpenPhysicalDispute>, threshold: u8) -> Result<()>{
        PhysicalTransaction::open_dispute(ctx, threshold)
    }

    pub fn close_dispute(ctx: Context<ClosePhysicalDispute>) -> Result<()>{
        PhysicalTransaction::close_dispute(ctx)
    }

    /////////////////////////////////////////////////
    /// REVIEW RELATED
    
    pub fn leave_review(ctx: Context<LeaveReview>, rating: u8) -> Result<()>{
        PhysicalTransaction::leave_review(ctx, rating)
    }

}