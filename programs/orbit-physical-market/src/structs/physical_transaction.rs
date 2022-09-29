use anchor_lang::prelude::*;
use market_accounts::structs::TransactionReviews;
use transaction::transaction_struct::OrbitTransaction;

#[account]
pub struct PhysicalTransaction{
    pub metadata: OrbitTransaction, // 32 * 3 + 5?
    pub shipping: [u8; 64],
}