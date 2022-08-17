use anchor_lang::{
    prelude::*,
    AccountsClose
};
use crate::{structs::physical_product::PhysicalProduct, errors::PhysicalMarketErrors};
use product::{
    product_trait::OrbitProductTrait,
    product_struct::OrbitProduct
};
use market_accounts::structs::market_account::OrbitMarketAccount;


//////////////////////////////////////////////////////////////////////////////
/// DEFAULT PRODUCT TRAIT


// todo:
//      add to catalog
#[derive(Accounts)]
pub struct ListPhysicalProduct<'info>{

    #[account(
        init,
        space = 1000, // 106 + 8. leave room for adjustment during launch
        payer = seller_wallet
    )]
    pub new_product: Account<'info, PhysicalProduct>,

    pub seller_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut,
        address = seller_account.wallet
    )]
    pub seller_wallet: Signer<'info>,
    
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct UnlistPhysicalProduct<'info>{
    #[account(
        mut,
        constraint = phys_product.metadata.seller == seller_account.key()
    )]
    pub phys_product: Account<'info, PhysicalProduct>,

    pub seller_account: Account<'info, OrbitMarketAccount>,

    #[account(
        mut,
        address = seller_account.wallet
    )]
    pub seller_wallet: Signer<'info>,

    pub system_program: Program<'info, System>
}

impl<'a, 'b> OrbitProductTrait<'a, 'b, ListPhysicalProduct<'a>, UnlistPhysicalProduct<'b>> for PhysicalProduct{
    fn list(ctx: Context<ListPhysicalProduct>, prod: OrbitProduct) -> Result<()>{
        if prod.seller != ctx.accounts.seller_account.key() {
            return err!(PhysicalMarketErrors::InvalidSellerForListing)
        }
        ctx.accounts.new_product.metadata = prod;
        Ok(())
    }

    fn unlist(ctx: Context<UnlistPhysicalProduct>) -> Result<()>{
        // closes account
        // returns error if errors
        ctx.accounts.phys_product.close(ctx.accounts.seller_wallet.to_account_info())
    }
}

//////////////////////////////////////////////////////////////////////////////
/// PHYSICAL PRODUCT SPECIFIC FUNCTIONALITIES

#[derive(Accounts)]
pub struct UpdateProductField<'info>{
    #[account(
        mut,
        constraint = phys_product.metadata.seller == market_account.key()
    )]
    pub phys_product: Account<'info, PhysicalProduct>,

    #[account(
        has_one = master_pubkey
    )]
    pub market_account: Account<'info, OrbitMarketAccount>,

    pub master_pubkey: Signer<'info>,
}

pub fn update_quantity_handler(ctx: Context<UpdateProductField>, qnt: u32) -> Result<()>{
    ctx.accounts.phys_product.quantity = qnt;
    if qnt == 0{
        ctx.accounts.phys_product.metadata.available = false;
    }
    Ok(())
}

pub fn update_price_handler(ctx: Context<UpdateProductField>, price: u64) -> Result<()>{
    ctx.accounts.phys_product.metadata.price = price;
    Ok(())
}