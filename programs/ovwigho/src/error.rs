use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The collection has already been initialized.")]
    CollectionAlreadyInitialized,
    #[msg("The collection has not been initialized.")]
    CollectionNotInitialized,
    #[msg("The asset has already been initialized.")]
    AssetAlreadyInitialized,
    #[msg("Not enough cNFTs burned")]
    NotEnoughBurns,
}
