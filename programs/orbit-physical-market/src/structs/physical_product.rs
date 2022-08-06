use product::product_struct::OrbitProduct;
use anchor_lang::prelude::*;

#[account]
pub struct PhysicalProduct{
    pub metadata: OrbitProduct,
    pub quantity: i8, // quantity per purchase
}