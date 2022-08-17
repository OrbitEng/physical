use anchor_lang::prelude::*;
use product::product_struct::OrbitProduct;
use product::product_trait::OrbitProductTrait;
use transaction::transaction_trait::OrbitTransactionTrait;
use dispute::OrbitDisputableTrait;

pub mod structs;
pub mod accessors;
pub mod errors;

pub use structs::*;
pub use accessors::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
pub mod orbit_physical_market {

    use super::*;

    // pub fn initialize_phys_market(_ctx: Context<Initialize>) -> Result<()>{
    //     Ok(())
    // }
    
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

    pub fn open_transaction(ctx: Context<OpenPhysicalTransaction>, price: u64) -> Result<()>{
        PhysicalTransaction::open(ctx, price)
    }

    pub fn close_transaction(ctx: Context<ClosePhysicalTransaction>) -> Result<()>{
        PhysicalTransaction::close(ctx)
    }

    pub fn open_dispute(ctx: Context<OpenPhysicalDispute>, threshold: u8) -> Result<()>{
        PhysicalTransaction::open_dispute(ctx, threshold)
    }

    pub fn close_dispute(ctx: Context<ClosePhysicalDispute>) -> Result<()>{
        PhysicalTransaction::close_dispute(ctx)
    }

}

// #[derive(Accounts)]
// pub struct Initialize<'info>{
//     #[account(
//         seeds = [b"physical_auth"],
//         bump
//     )]
//     pub phys_auth: SystemAccount<'info>,

//     #[account(mut)]
//     pub payer: Signer<'info>,

//     pub system_program: Program<'info, System>
// }