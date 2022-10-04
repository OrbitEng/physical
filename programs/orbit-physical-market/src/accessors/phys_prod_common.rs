use anchor_lang::{
    prelude::*,
    AccountsClose
};

use orbit_catalog::{
    structs::{
        OrbitModCatalogStruct,
        OrbitVendorCatalog
    },
    cpi::{
        accounts::{
            EditModCatalog,
            ModifyVendorCatalog
        },
        edit_mod_catalog, list_product, unlist_product
    }, program::OrbitCatalog
};
use crate::{structs::physical_product::PhysicalProduct, errors::PhysicalMarketErrors, program::OrbitPhysicalMarket};
use product::{
    product_trait::OrbitProductTrait,
    product_struct::OrbitProduct,
    CommonProdUtils
};

//////////////////////////////////////////////////////////////////////////////
/// DEFAULT PRODUCT TRAIT

#[derive(Accounts)]
#[instruction(prod_in: OrbitProduct)]
pub struct ListPhysicalProduct<'info>{

    #[account(
        init,
        space = 300, // 106 + 8. leave room for adjustment during launch
        payer = seller_wallet,

        seeds = [
            b"commission_product",
            vendor_catalog.key().as_ref(),
            &[prod_in.index]
        ],
        bump
    )]
    pub phys_product: Box<Account<'info, PhysicalProduct>>,

    #[account(
        address = phys_product.metadata.owner_catalog
    )]
    pub vendor_catalog: Account<'info, OrbitVendorCatalog>,

    #[account(
        mut,
        address = vendor_catalog.catalog_owner
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
    #[account(mut)]
    pub phys_product: Box<Account<'info, PhysicalProduct>>,

    #[account(
        address = phys_product.metadata.owner_catalog
    )]
    pub vendor_catalog: Account<'info, OrbitVendorCatalog>,

    #[account(
        mut,
        address = vendor_catalog.catalog_owner
    )]
    pub seller_wallet: Signer<'info>,

    pub catalog_program: Program<'info, OrbitCatalog>,
}

impl<'a, 'b> OrbitProductTrait<'a, 'b, ListPhysicalProduct<'a>, UnlistPhysicalProduct<'b>> for PhysicalProduct{
    fn list(ctx: Context<ListPhysicalProduct>, prod: OrbitProduct) -> Result<()>{
        if prod.owner_catalog != ctx.accounts.vendor_catalog.key() {
            return err!(PhysicalMarketErrors::InvalidSellerForListing)
        }
        
        list_product(
            CpiContext::new(
                ctx.accounts.catalog_program.to_account_info(),
                ModifyVendorCatalog{
                    vendor_catalog: ctx.accounts.vendor_catalog.to_account_info(),
                    wallet: ctx.accounts.seller_wallet.to_account_info()
                }
            ), 
            prod.index.clone()
        ).expect("could not list product, index is taken");

        ctx.accounts.phys_product.metadata = prod;
        ctx.accounts.phys_product.quantity = 0;

        match ctx.bumps.get("market_auth"){
            Some(auth_bump) => edit_mod_catalog(
                CpiContext::new_with_signer(
                    ctx.accounts.catalog_program.to_account_info(),
                    EditModCatalog {
                        catalog: ctx.accounts.recent_catalog.to_account_info(),
                        product: ctx.accounts.phys_product.to_account_info(),
                        caller_auth: ctx.accounts.market_auth.to_account_info()
                    },
                    &[&[b"market_auth", &[*auth_bump]]])
            ),
            None => err!(PhysicalMarketErrors::InvalidAuthBump)
        }
    }

    fn unlist(ctx: Context<UnlistPhysicalProduct>) -> Result<()>{
        unlist_product(
            CpiContext::new(
                ctx.accounts.catalog_program.to_account_info(),
                ModifyVendorCatalog{
                    vendor_catalog: ctx.accounts.vendor_catalog.to_account_info(),
                    wallet: ctx.accounts.seller_wallet.to_account_info()
                }
            ),
            ctx.accounts.phys_product.metadata.index
        ).expect("could not update catalog");
        ctx.accounts.phys_product.close(ctx.accounts.seller_wallet.to_account_info())
    }
}

//////////////////////////////////////////////////////////////////////////////
/// PHYSICAL PRODUCT SPECIFIC FUNCTIONALITIES

#[derive(Accounts, CommonProdUtils)]
pub struct UpdateProductField<'info>{
    #[account(mut)]
    pub phys_product: Account<'info, PhysicalProduct>,

    #[account(
        address = phys_product.metadata.owner_catalog
    )]
    pub vendor_catalog: Account<'info, OrbitVendorCatalog>,

    #[account(
        mut,
        address = vendor_catalog.catalog_owner
    )]
    pub seller_wallet: Signer<'info>,
}

pub fn update_quantity_handler(ctx: Context<UpdateProductField>, qnt: u32) -> Result<()>{
    ctx.accounts.phys_product.quantity = qnt;
    Ok(())
}