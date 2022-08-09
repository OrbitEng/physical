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


#[derive(Accounts)]
pub struct ListPhysicalProduct<'info>{

    #[account(
        init,
        space = 47, // base is 47 rn (39 + 8). increase this to a proper amount for wiggle room for additions
        payer = payer
    )]
    pub new_product: Account<'info, PhysicalProduct>,

    // todo: link seller with payer somehow? maybe do a check eg: only the seller can fund their own products
    pub seller: Account<'info, OrbitMarketAccount>,

    #[account(
        mut,
    )]
    pub payer: Signer<'info>,

    #[account(
        address = seller.master_pubkey
    )]
    pub authority: Signer<'info>,
    
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

    pub market_account: Account<'info, OrbitMarketAccount>,

    #[account(
        address = market_account.master_pubkey
    )]
    pub authority: Signer<'info>,

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