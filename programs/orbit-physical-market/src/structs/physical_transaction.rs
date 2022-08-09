use anchor_lang::prelude::*;
use transaction::transaction_struct::OrbitTransaction;

// technically an AccountInfo with extra sugar
// impls AnchorSerialize, AnchorDeserialize, mem::LEN, Clone, and Deref I believe
#[account]
pub struct PhysicalTransaction{
    pub metadata: OrbitTransaction, // 32 * 3 + 5?
    pub escrow_account: Pubkey,
}