use product::product_struct::OrbitProduct;
use anchor_lang::prelude::*;

// disc a81c2d2c [168, 28, 45, 44, 211, 241, 238, 140]
#[account]
pub struct PhysicalProduct{
    pub metadata: OrbitProduct, // 38 as of beta
    pub quantity: u32, // quantity per purchase // 4
}