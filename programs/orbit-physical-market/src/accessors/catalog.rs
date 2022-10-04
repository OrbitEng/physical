use anchor_lang::prelude::*;
use crate::errors::PhysicalMarketErrors;
use orbit_catalog::{cpi::{
    accounts::CreateMarketCatalog,
    init_market_catalog
}, program::OrbitCatalog};

#[derive(Accounts)]
pub struct CreatePhysRecentCatalog<'info>{
    #[account(
        seeds = [
            b"recent_catalog"
        ],
        bump
    )]
    pub catalog: SystemAccount<'info>,

    #[account(
        seeds = [
            b"market_auth"
        ],
        bump
    )]
    pub market_auth: SystemAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub catalog_program: Program<'info, OrbitCatalog>,

    pub system_program: Program<'info, System>
}

pub fn recent_phys_catalog_handler(ctx: Context<CreatePhysRecentCatalog>) -> Result<()>{
    match ctx.bumps.get("market_auth"){
        Some(auth_bump) => init_market_catalog(
            CpiContext::new_with_signer(
                ctx.accounts.catalog_program.to_account_info(),
                CreateMarketCatalog {
                    catalog: ctx.accounts.catalog.to_account_info(),
                    payer: ctx.accounts.payer.to_account_info(),
                    system_program: ctx.accounts.system_program.to_account_info()
                },
                &[&[b"market_auth", &[*auth_bump]]]
            )
        ),
        None => err!(PhysicalMarketErrors::InvalidAuthBump)
    }
    
}