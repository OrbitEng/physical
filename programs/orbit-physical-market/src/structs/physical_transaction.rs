use anchor_lang::prelude::*;
use transaction::transaction_trait::OrbitTransactionTrait;

#[account]
pub struct PhysicalTransaction{

}

impl OrbitTransactionTrait for PhysicalTransaction{}