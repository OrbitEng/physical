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

    pub fn update_currency(ctx: Context<UpdateProductField>, currency: Pubkey) -> Result<()>{
        update_currency_handler(ctx, currency)
    }

    //////////////////////////////////////////////
    /// TRANSACTION HANDLERS

    /// SOL
    pub fn open_transaction_sol(ctx: Context<OpenPhysicalTransactionSol>, price: u64) -> Result<()>{
        PhysicalTransaction::open_sol(ctx, price)
    }

    pub fn close_transaction_sol(ctx: Context<ClosePhysicalTransactionSol>) -> Result<()>{
        PhysicalTransaction::close_sol(ctx)
    }

    pub fn fund_escrow_sol(ctx: Context<FundEscrowSol>) -> Result<()>{
        PhysicalTransaction::fund_escrow_sol(ctx)
    }

    /// SPL
    pub fn open_transaction_spl(ctx: Context<OpenPhysicalTransactionSpl>, price: u64) -> Result<()>{
        PhysicalTransaction::open_spl(ctx, price)
    }

    pub fn close_transaction_spl(ctx: Context<ClosePhysicalTransactionSpl>) -> Result<()>{
        PhysicalTransaction::close_spl(ctx)
    }

    pub fn fund_escrow_spl(ctx: Context<FundEscrowSpl>) -> Result<()>{
        PhysicalTransaction::fund_escrow_spl(ctx)
    }

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

// #[derive(Accounts)]
// pub struct Initialize<'info>{
//     #[accounts(
//         init,
//         seeds = [
//             b"recent_catalog"
//         ],
//         bump,
//         payer = payer
//     )]
//     pub catalog: SystemAccount
// }