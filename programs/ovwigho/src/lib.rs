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

declare_id!("87SMpWyhZpRWKeRxaMNMwwjDwMBLhYLSAbPjkPJMRdii");

#[program]
pub mod ovwigho {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        max_depth: u32,
        max_buffer_size: u32,
        cnft_args: CreateCnftCollectionArgs,
        nft_args: CreateNftCollectionArgs,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, max_depth, max_buffer_size, cnft_args, nft_args)?;
        Ok(())
    }
}
