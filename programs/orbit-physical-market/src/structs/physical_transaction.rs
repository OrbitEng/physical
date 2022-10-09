use anchor_lang::prelude::*;
use orbit_transaction::transaction_struct::OrbitTransaction;

#[account]
pub struct PhysicalTransaction{
    pub metadata: OrbitTransaction, // 32 * 3 + 5?
    pub shipping: [u8; 64],
}