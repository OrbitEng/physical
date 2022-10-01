use anchor_lang::{
    prelude::*,
    AccountsClose
};

use orbit_catalog::{
    catalog_struct::OrbitModCatalogStruct,
    cpi::{
        accounts::EditModCatalog,
        edit_mod_catalog
    }, program::OrbitCatalog
};
use crate::{structs::physical_product::PhysicalProduct, errors::PhysicalMarketErrors, program::OrbitPhysicalMarket};
use product::{
    product_trait::OrbitProductTrait,
    product_struct::OrbitProduct,
    CommonProdUtils
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
        space = 300, // 106 + 8. leave room for adjustment during launch
        payer = seller_wallet
    )]
    pub new_product: Box<Account<'info, PhysicalProduct>>,

    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,

    #[account(
        mut,
        address = seller_account.wallet
    )]
    pub seller_wallet: Signer<'info>,
    
    pub system_program: Program<'info, System>,

    #[account(
        mut,
        seeds = [
            b"recent_catalog"
        ],
        bump
    )]
    pub recent_catalog: Account<'info, OrbitModCatalogStruct>,

    #[account(
        seeds = [
            b"market_auth"
        ],
        bump
    )]
    pub market_auth: SystemAccount<'info>,

    pub catalog_program: Program<'info, OrbitCatalog>,

    pub phys_program: Program<'info, OrbitPhysicalMarket>,
}

#[derive(Accounts)]
pub struct UnlistPhysicalProduct<'info>{
    #[account(
        mut,
        constraint = phys_product.metadata.seller == seller_account.key()
    )]
    pub phys_product: Box<Account<'info, PhysicalProduct>>,

    pub seller_account: Box<Account<'info, OrbitMarketAccount>>,

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

        match ctx.bumps.get("market_auth"){
            Some(auth_bump) => edit_mod_catalog(
                CpiContext::new_with_signer(
                    ctx.accounts.catalog_program.to_account_info(),
                    EditModCatalog {
                        catalog: ctx.accounts.recent_catalog.to_account_info(),
                        product: ctx.accounts.new_product.to_account_info(),
                        caller_auth: ctx.accounts.market_auth.to_account_info()
                    },
                    &[&[b"market_auth", &[*auth_bump]]])
            ),
            None => err!(PhysicalMarketErrors::InvalidAuthBump)
        }
    }

    fn unlist(ctx: Context<UnlistPhysicalProduct>) -> Result<()>{
        // closes account
        // returns error if errors
        ctx.accounts.phys_product.close(ctx.accounts.seller_wallet.to_account_info())
    }
}

//////////////////////////////////////////////////////////////////////////////
/// PHYSICAL PRODUCT SPECIFIC FUNCTIONALITIES

#[derive(Accounts, CommonProdUtils)]
pub struct UpdateProductField<'info>{
    #[account(
        mut,
        constraint = phys_product.metadata.seller == market_account.key()
    )]
    pub phys_product: Account<'info, PhysicalProduct>,

    #[account(
        seeds = [
            b"orbit_account",
            wallet.key().as_ref()
        ],
        bump,
        seeds::program = market_accounts::ID
    )]
    pub market_account: Account<'info, OrbitMarketAccount>,

    #[account(
        address = market_account.wallet
    )]
    pub wallet: Signer<'info>,
}

pub fn update_quantity_handler(ctx: Context<UpdateProductField>, qnt: u32) -> Result<()>{
    ctx.accounts.phys_product.quantity = qnt;
    if qnt == 0{
        ctx.accounts.phys_product.metadata.available = false;
    }
    Ok(())
}