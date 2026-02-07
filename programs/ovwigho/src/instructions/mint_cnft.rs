use anchor_lang::prelude::*;
use mpl_account_compression::ID as MPL_ACCOUNT_COMPRESSION_ID;
use mpl_bubblegum::instructions::MintV2CpiBuilder;
use mpl_bubblegum::types::{MetadataArgsV2, TokenStandard};
use mpl_bubblegum::ID as BUBBLEGUM_ID;
use mpl_noop::ID as MPL_NOOP_ID;

use crate::{error::ErrorCode, Config};

#[derive(Accounts)]
pub struct MintcNFT<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    pub authority: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"config", authority.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, Config>>,

    /// CHECK: Collection Account that will be checked by the Bubblegum Program
    #[account(
        mut,
        constraint = !cnft_collection.data_is_empty() @ ErrorCode::CollectionNotInitialized,
        address = config.cnft_collection,
    )]
    pub cnft_collection: UncheckedAccount<'info>,

    /// CHECK: Tree Config checks will be performed by the Bubblegum Program
    #[account(mut)]
    pub tree_config: UncheckedAccount<'info>,
    /// CHECK: Merkle Tree account that will be checked by the Bubblegum Program
    #[account(mut)]
    pub merkle_tree: UncheckedAccount<'info>,
    /// CHECK: MPL Core CPI Signer account that will be checked by the Bubblegum Program
    pub mpl_core_cpi_signer: UncheckedAccount<'info>,
    /// CHECK: MPL NOOP Program checked by the corresponding address
    #[account(address = MPL_NOOP_ID)]
    pub log_wrapper: UncheckedAccount<'info>,
    /// CHECK: Bubblegum Program checked by the corresponding address
    #[account(address = BUBBLEGUM_ID)]
    pub bubblegum_program: UncheckedAccount<'info>,
    /// CHECK: MPL Account Compression Program checked by the corresponding address
    #[account(address = MPL_ACCOUNT_COMPRESSION_ID)]
    pub compression_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    /// CHECK: This is the ID of the Metaplex Core program
    #[account(address = mpl_core::ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

impl<'info> MintcNFT<'info> {
    pub fn mint_cnft(&mut self, name: String, uri: String, symbol: String) -> Result<()> {
        let seeds = &[
            &b"config"[..],
            &self.authority.key.as_ref(),
            &[self.config.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        MintV2CpiBuilder::new(&self.bubblegum_program.to_account_info())
            .tree_config(&self.tree_config.to_account_info())
            .leaf_owner(&self.player.to_account_info())
            .leaf_delegate(Some(&self.player))
            .merkle_tree(&self.merkle_tree.to_account_info())
            .payer(&self.player.to_account_info())
            .tree_creator_or_delegate(Some(&self.config.to_account_info()))
            .core_collection(Some(&self.cnft_collection.to_account_info()))
            .collection_authority(Some(&self.config.to_account_info()))
            .mpl_core_cpi_signer(Some(&self.mpl_core_cpi_signer.to_account_info()))
            .log_wrapper(&self.log_wrapper.to_account_info())
            .compression_program(&self.compression_program.to_account_info())
            .system_program(&self.system_program.to_account_info())
            .mpl_core_program(&self.mpl_core_program.to_account_info())
            .metadata(MetadataArgsV2 {
                name,
                symbol,
                uri,
                seller_fee_basis_points: 0,
                primary_sale_happened: false,
                is_mutable: false,
                token_standard: Some(TokenStandard::NonFungible),
                creators: vec![],
                collection: Some(self.cnft_collection.key()),
            })
            .invoke_signed(signer_seeds)?;

        Ok(())
    }
}

pub fn handler(ctx: Context<MintcNFT>, name: String, uri: String, symbol: String) -> Result<()> {
    ctx.accounts.mint_cnft(name, uri, symbol)?;

    Ok(())
}
