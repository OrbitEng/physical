use anchor_lang::{
    prelude::*,
    AccountsClose
};
use crate::structs::physical_product::PhysicalProduct;
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
        payer = payer
    )]
    pub new_product: Account<'info, PhysicalProduct>,

    #[account(
        has_one = master_pubkey
    )]
    pub seller: Account<'info, OrbitMarketAccount>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub master_pubkey: Signer<'info>,
    
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct UnlistPhysicalProduct<'info>{
    #[account(
        mut,
        constraint = phys_product.metadata.seller == market_account.key()
    )]
    pub phys_product: Account<'info, PhysicalProduct>,

    // they should only be able to close and send funds to wallet linked.
    // i dont think not having constraint would raise issues but
    // for scams/ just to be safe I want this constraint. for now.
    #[account(
        mut,
        address = market_account.wallet
    )]
    pub destination_wallet: AccountInfo<'info>,

    #[account(
        has_one = master_pubkey
    )]
    pub market_account: Account<'info, OrbitMarketAccount>,

    pub master_pubkey: Signer<'info>,

    pub system_program: Program<'info, System>
}

impl<'a, 'b> OrbitProductTrait<'a, 'b, ListPhysicalProduct<'a>, UnlistPhysicalProduct<'b>> for PhysicalProduct{
    fn list(ctx: Context<ListPhysicalProduct>, prod: OrbitProduct) -> Result<()>{
        ctx.accounts.new_product.metadata = prod;
        Ok(())
    }

    fn unlist(ctx: Context<UnlistPhysicalProduct>) -> Result<()>{
        // closes account
        // returns error if errors
        ctx.accounts.phys_product.close(ctx.accounts.destination_wallet.clone()).map_err(|e| e)
    }
}

//////////////////////////////////////////////////////////////////////////////
/// PHYSICAL PRODUCT SPECIFIC FUNCTIONALITIES

#[derive(Accounts)]
pub struct UpdateQuantity<'info>{
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

pub fn update_quantity_handler(ctx: Context<UpdateQuantity>, qnt: u32) -> Result<()>{
    ctx.accounts.phys_product.quantity = qnt;
    if qnt == 0{
        ctx.accounts.phys_product.metadata.available = false;
    }
    Ok(())
}