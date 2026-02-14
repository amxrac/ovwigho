use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Config {
    pub authority: Pubkey,
    pub cnft_collection: Pubkey,
    pub nft_collection: Pubkey,
    pub merkle_tree: Pubkey,
    pub total_cnfts_minted: u32,
    pub total_nfts_minted: u32,
    pub bump: u8,
}
