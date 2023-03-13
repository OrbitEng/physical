use anchor_lang::prelude::*;

#[error_code]
pub enum PhysicalMarketErrors{
    #[msg("dispute already opened")]
    DisputeExists,
    #[msg("could not find bump for escrow")]
    InvalidEscrowBump,
    #[msg("could not find bump for market authority")]
    InvalidAuthBump,
    #[msg("invalid seller for listing")]
    InvalidSellerForListing,
    #[msg("invalid reflink passed")]
    InvalidReflink,
    #[msg("not a transaction participant")]
    InvalidTransactionInvoker,
    #[msg("Please confirm delivery first")]
    DidNotConfirmDelivery,
}