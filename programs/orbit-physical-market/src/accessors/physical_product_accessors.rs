use anchor_lang::prelude::*;
use crate::structs::physical_product::PhysicalProduct;
use product::{
    product_trait::OrbitProductTrait,
    product_struct::OrbitProduct
};


#[derive(Accounts)]
pub struct ListPhysicalProduct<'info>{

    // #[account(
    //     init, 
    // )]


    
    
    pub system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct UnlistPhysicalProduct<'info>{
    pub system_program: Program<'info, System>
}

impl OrbitProductTrait for PhysicalProduct{
    fn list<ListPhysicalProduct>(ctx: Context<ListPhysicalProduct>, prod: OrbitProduct) -> Result<()>{
        Ok(())
    }

    fn unlist<UnlistPhysicalProduct>(ctx: Context<UnlistPhysicalProduct>) -> Result<()>{
        Ok(())
    }
}