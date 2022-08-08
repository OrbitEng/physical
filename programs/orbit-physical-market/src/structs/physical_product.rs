use product::product_struct::OrbitProduct;
use anchor_lang::prelude::*;

#[account]
pub struct PhysicalProduct{
    pub metadata: OrbitProduct, // 38 as of beta
    pub quantity: i8, // quantity per purchase // 1
}