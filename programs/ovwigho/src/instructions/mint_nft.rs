use anchor_lang::prelude::*;
use mpl_core::{
    instructions::CreateV2CpiBuilder,
    types::{Attribute, Attributes, Plugin, PluginAuthority, PluginAuthorityPair},
    ID as CORE_PROGRAM_ID,
};

use crate::{error::ErrorCode, Config, PlayerProgress};

#[derive(Accounts)]
pub struct MintNFT<'info> {
    #[account(mut)]
    pub player: Signer<'info>,

    pub authority: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [b"config", authority.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(
        mut,
        seeds = [b"player", player.key().as_ref()],
        bump = player_progress.bump,
        constraint = player_progress.total_cnfts_burned == 5 @ ErrorCode::NotEnoughBurns

    )]
    pub player_progress: Box<Account<'info, PlayerProgress>>,

    /// CHECK: Collection Account that will be checked by core
    #[account(
        mut,
        constraint = !nft_collection.data_is_empty() @ ErrorCode::CollectionNotInitialized,
        address = config.nft_collection,
    )]
    pub nft_collection: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = asset.data_is_empty() @ ErrorCode::AssetAlreadyInitialized
    )]
    pub asset: Signer<'info>,

    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: This will also be checked by core
    pub core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> MintNFT<'info> {
    pub fn mint_nft(&mut self, name: String, uri: String) -> Result<()> {
        let seeds = &[
            &b"config"[..],
            &self.authority.key.as_ref(),
            &[self.config.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        CreateV2CpiBuilder::new(&self.core_program.to_account_info())
            .asset(&self.asset.to_account_info())
            .collection(Some(&self.nft_collection.to_account_info()))
            .authority(Some(&self.config.to_account_info()))
            .payer(&self.player.to_account_info())
            .owner(Some(&self.player.to_account_info()))
            .update_authority(None)
            .system_program(&self.system_program.to_account_info())
            .name(name)
            .uri(uri)
            .plugins(vec![PluginAuthorityPair {
                plugin: Plugin::Attributes(Attributes {
                    attribute_list: vec![
                        Attribute {
                            key: "Player".to_string(),
                            value: self.player.key().to_string(),
                        },
                        Attribute {
                            key: "Collection".to_string(),
                            value: self.nft_collection.key().to_string(),
                        },
                    ],
                }),
                authority: Some(PluginAuthority::Address {
                    address: self.config.key(),
                }),
            }])
            .external_plugin_adapters(vec![])
            .invoke_signed(signer_seeds)?;

        Ok(())
    }

    pub fn record_mint(&mut self) -> Result<()> {
        self.config.total_nfts_minted += 1;

        self.player_progress.total_cnfts_burned = 0;
        self.player_progress.total_nfts_minted += 1;
        Ok(())
    }
}

pub fn handler(ctx: Context<MintNFT>, name: String, uri: String) -> Result<()> {
    ctx.accounts.mint_nft(name, uri)?;

    ctx.accounts.record_mint()?;
    Ok(())
}
