use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PlayerProgress {
    pub player: Pubkey,
    pub authority: Pubkey,
    pub total_cnfts_burned: u32,
    pub total_nfts_minted: u32,
    pub bump: u8,
}
