use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("The collection has already been initialized.")]
    CollectionAlreadyInitialized,
}
