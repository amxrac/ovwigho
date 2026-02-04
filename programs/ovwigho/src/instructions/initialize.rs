use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use mpl_account_compression::ID as MPL_ACCOUNT_COMPRESSION_ID;
use mpl_bubblegum::{instructions::CreateTreeConfigV2CpiBuilder, ID as BUBBLEGUM_ID};
use mpl_core::{
    instructions::CreateCollectionV2CpiBuilder,
    types::{BubblegumV2, PluginAuthorityPair},
    ID as CORE_PROGRAM_ID,
};
use mpl_noop::ID as MPL_NOOP_ID;

use crate::{error::ErrorCode, Config};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateCnftCollectionArgs {
    pub name: String,
    pub uri: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct CreateNftCollectionArgs {
    pub name: String,
    pub uri: String,
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        seeds = [b"config", authority.key().as_ref()],
        bump,
        space = Config::DISCRIMINATOR.len() + Config::INIT_SPACE,
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(
        mut,
        constraint = cnft_collection.data_is_empty() @ ErrorCode::CollectionAlreadyInitialized
    )]
    pub cnft_collection: Signer<'info>,

    #[account(
        mut,
        constraint = nft_collection.data_is_empty() @ ErrorCode::CollectionAlreadyInitialized
    )]
    pub nft_collection: Signer<'info>,

    /// CHECK: Tree Config checks will be performed by the Bubblegum Program
    #[account(mut)]
    pub tree_config: UncheckedAccount<'info>,
    /// CHECK: Unitialized Merkle Tree Account. Initialization will be performed by the Bubblegum Program
    #[account(mut)]
    pub merkle_tree: UncheckedAccount<'info>,
    /// CHECK: MPL NOOP Program checked by the corresponding address
    #[account(address = MPL_NOOP_ID)]
    pub log_wrapper: UncheckedAccount<'info>,
    /// CHECK: Bubblegum Program checked by the corresponding address
    #[account(address = BUBBLEGUM_ID)]
    pub bubblegum_program: UncheckedAccount<'info>,
    /// CHECK: MPL Account Compression Program checked by the corresponding address
    #[account(address = MPL_ACCOUNT_COMPRESSION_ID)]
    pub compression_program: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    #[account(address = CORE_PROGRAM_ID)]
    /// CHECK: This will also be checked by core
    pub core_program: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.config.set_inner(Config {
            authority: self.authority.key(),
            cnft_collection: self.cnft_collection.key(),
            nft_collection: self.nft_collection.key(),
            merkle_tree: self.merkle_tree.key(),
            total_cnfts_minted: 0,
            total_nfts_minted: 0,
            bump: bumps.config,
        });

        Ok(())
    }

    pub fn init_merkle_tree(&mut self, max_depth: u32, max_buffer_size: u32) -> Result<()> {
        let seeds = &[
            &b"config"[..],
            &self.authority.key.as_ref(),
            &[self.config.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let bubblegum_program = &self.bubblegum_program.to_account_info();
        let tree_config = &self.tree_config.to_account_info();
        let merkle_tree = &self.merkle_tree.to_account_info();
        let tree_creator = &self.config.to_account_info();
        let payer = &self.authority.to_account_info();
        let log_wrapper = &self.log_wrapper.to_account_info();
        let compression_program = &self.compression_program.to_account_info();
        let system_program = &self.system_program.to_account_info();

        CreateTreeConfigV2CpiBuilder::new(bubblegum_program)
            .tree_config(tree_config)
            .merkle_tree(merkle_tree)
            .payer(payer)
            .tree_creator(Some(tree_creator))
            .log_wrapper(log_wrapper)
            .compression_program(compression_program)
            .system_program(system_program)
            .max_depth(max_depth)
            .max_buffer_size(max_buffer_size)
            .public(false)
            .invoke_signed(signer_seeds)?;

        Ok(())
    }

    pub fn create_cnft_collection(
        &mut self,
        bumps: &InitializeBumps,
        args: CreateCnftCollectionArgs,
    ) -> Result<()> {
        let seeds = &[
            &b"config"[..],
            &self.authority.key.as_ref(),
            &[bumps.config],
        ];
        let signer_seeds = &[&seeds[..]];

        CreateCollectionV2CpiBuilder::new(&self.core_program.to_account_info())
            .collection(&self.cnft_collection.to_account_info())
            .update_authority(Some(&self.config.to_account_info()))
            .payer(&self.authority.to_account_info())
            .system_program(&self.system_program.to_account_info())
            .name(args.name)
            .uri(args.uri)
            .plugins(vec![PluginAuthorityPair {
                plugin: mpl_core::types::Plugin::BubblegumV2(BubblegumV2 {}),
                authority: None,
            }])
            .external_plugin_adapters(vec![])
            .invoke_signed(signer_seeds)?;

        Ok(())
    }

    pub fn create_nft_collection(
        &mut self,
        args: CreateNftCollectionArgs,
        bumps: &InitializeBumps,
    ) -> Result<()> {
        let seeds = &[
            &b"config"[..],
            &self.authority.key.as_ref(),
            &[bumps.config],
        ];
        let signer_seeds = &[&seeds[..]];

        CreateCollectionV2CpiBuilder::new(&self.core_program.to_account_info())
            .collection(&self.nft_collection.to_account_info())
            .update_authority(Some(&self.config.to_account_info()))
            .payer(&self.authority.to_account_info())
            .system_program(&self.system_program.to_account_info())
            .name(args.name)
            .uri(args.uri)
            .plugins(vec![])
            .invoke_signed(signer_seeds)?;

        Ok(())
    }
}

pub fn handler(
    ctx: Context<Initialize>,
    max_depth: u32,
    max_buffer_size: u32,
    cnft_args: CreateCnftCollectionArgs,
    nft_args: CreateNftCollectionArgs,
) -> Result<()> {
    ctx.accounts.initialize(&ctx.bumps)?;
    ctx.accounts.init_merkle_tree(max_depth, max_buffer_size)?;
    ctx.accounts.create_cnft_collection(&ctx.bumps, cnft_args)?;
    ctx.accounts.create_nft_collection(nft_args, &ctx.bumps)?;
    Ok(())
}
