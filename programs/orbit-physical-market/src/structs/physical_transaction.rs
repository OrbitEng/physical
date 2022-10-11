use anchor_lang::prelude::*;
use orbit_transaction::transaction_struct::OrbitTransactionStruct;

#[account]
pub struct PhysicalTransaction{
    pub metadata: OrbitTransactionStruct, // 32 * 3 + 5?
    pub shipping: [u8; 64],
}