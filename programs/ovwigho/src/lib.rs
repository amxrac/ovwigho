#![allow(unexpected_cfgs)]
#![allow(deprecated)]

pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use instructions::*;
pub use state::*;

declare_id!("Gf44xpXRCeZAy4kWYDLrsVDAGQhvKC3Y5DqtJBeojzZ4");

#[program]
pub mod ovwigho {
    use crate::instructions::mint_cnft::MintcNFT;

    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        max_depth: u32,
        max_buffer_size: u32,
        cnft_args: CreateCnftCollectionArgs,
        nft_args: CreateNftCollectionArgs,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, max_depth, max_buffer_size, cnft_args, nft_args)
    }

    pub fn mint_cnft(
        ctx: Context<MintcNFT>,
        name: String,
        uri: String,
        symbol: String,
    ) -> Result<()> {
        instructions::mint_cnft::handler(ctx, name, uri, symbol)
    }

    pub fn burn_cnft<'info>(
        ctx: Context<'_, '_, 'info, 'info, BurncNFT<'info>>,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
        asset_data_hash: [u8; 32],
        flags: u8,
    ) -> Result<()> {
        instructions::burn_cnft::handler(
            ctx,
            root,
            data_hash,
            creator_hash,
            nonce,
            index,
            asset_data_hash,
            flags,
        )
    }

    pub fn mint_nft(ctx: Context<MintNFT>, name: String, uri: String) -> Result<()> {
        instructions::mint_nft::handler(ctx, name, uri)
    }
}
