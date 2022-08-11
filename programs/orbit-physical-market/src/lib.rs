use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

pub mod structs;
pub mod accessors;

#[program]
pub mod orbit_physical_market {
    pub use structs::*;
    pub use accessors::*;

    use transaction::transaction_trait::OrbitTransactionTrait;
    use product::product_trait::OrbitProductTrait;

    pub fn initialize_phys_market(ctx: Context<Initialize>) -> Result<()>{
        Ok(())
    }
    
    pub fn open_transaction(ctx: Context<accessors::physical_transaction_accessors::OpenPhysicalTransaction>) -> bool{
        
    }

}

#[derive(Accounts)]
pub struct Initialize<'info>{
    #[accounts(
        init,
        space = 32,
        seeds = [b"phys_auth"],
        bump,
        payer = payer
    )]
    pub phys_auth: AccountInfo<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Account<'info, SystemProgram>
}