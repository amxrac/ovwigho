#![allow(unexpected_cfgs)]
#![allow(deprecated)]

pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;

pub use constants::*;
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

    pub fn mint_nft(ctx: Context<MintNFT>, name: String, uri: String) -> Result<()> {
        instructions::mint_nft::handler(ctx, name, uri)
    }
}
