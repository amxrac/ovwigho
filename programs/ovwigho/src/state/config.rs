use anchor_lang::prelude::*;

use crate::constants::{ANCHOR_DESCRIMINATOR_SIZE};

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub cnft_collection: Pubkey,
    pub nft_collection: Pubkey,
    pub merkle_tree: Pubkey,
    pub total_cnft_minted: u32,
    pub total_nft_minted: u32,
    pub bump: u8,
}
