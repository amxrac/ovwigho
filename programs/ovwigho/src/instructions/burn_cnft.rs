use anchor_lang::prelude::*;
use mpl_account_compression::ID as MPL_ACCOUNT_COMPRESSION_ID;
use mpl_bubblegum::instructions::BurnV2CpiBuilder;
use mpl_bubblegum::ID as BUBBLEGUM_ID;
use mpl_noop::ID as MPL_NOOP_ID;

use crate::{error::ErrorCode, Config, PlayerProgress};

#[derive(Accounts)]
pub struct BurncNFT<'info> {
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
        init_if_needed,
        payer = player,
        seeds = [b"player", player.key().as_ref()],
        bump,
        space = PlayerProgress::DISCRIMINATOR.len() + PlayerProgress::INIT_SPACE,

    )]
    pub player_progress: Box<Account<'info, PlayerProgress>>,

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

impl<'info> BurncNFT<'info> {
    pub fn init_player_progress(&mut self, bumps: &BurncNFTBumps) -> Result<()> {
        if self.player_progress.bump == 0 {
            self.player_progress.set_inner(PlayerProgress {
                player: self.player.key(),
                authority: self.authority.key(),
                total_cnfts_burned: 0,
                total_nfts_minted: 0,
                bump: bumps.player_progress,
            });
        }

        Ok(())
    }

    pub fn burn_cnft(
        &self,
        root: [u8; 32],
        data_hash: [u8; 32],
        creator_hash: [u8; 32],
        nonce: u64,
        index: u32,
        asset_data_hash: [u8; 32],
        flags: u8,
        remaining_accounts: &[AccountInfo<'info>],
    ) -> Result<()> {
        let seeds = &[
            &b"config"[..],
            &self.authority.key.as_ref(),
            &[self.config.bump],
        ];

        let signer_seeds = &[&seeds[..]];

        let bubblegum_program = &self.bubblegum_program.to_account_info();
        let tree_config = &self.tree_config.to_account_info();
        let merkle_tree = &self.merkle_tree.to_account_info();
        let log_wrapper = &self.log_wrapper.to_account_info();
        let compression_program = &self.compression_program.to_account_info();
        let system_program = &self.system_program.to_account_info();
        let authority = &self.config.to_account_info();
        let player = &self.player;
        let cnft_collection = &self.cnft_collection.to_account_info();
        let mpl_core_cpi_signer = &self.mpl_core_cpi_signer.to_account_info();
        let mpl_core_program = &self.mpl_core_program.to_account_info();

        let mut builder = BurnV2CpiBuilder::new(bubblegum_program);

        builder
            .tree_config(tree_config)
            .authority(Some(authority))
            .leaf_owner(player)
            .leaf_delegate(Some(player))
            .merkle_tree(merkle_tree)
            .core_collection(Some(cnft_collection))
            .mpl_core_cpi_signer(Some(mpl_core_cpi_signer))
            .log_wrapper(log_wrapper)
            .compression_program(compression_program)
            .mpl_core_program(mpl_core_program)
            .system_program(system_program)
            .root(root)
            .data_hash(data_hash)
            .creator_hash(creator_hash)
            .asset_data_hash(asset_data_hash)
            .flags(flags)
            .nonce(nonce)
            .index(index);

        for account in remaining_accounts {
            builder.add_remaining_accounts(&[(account, false, false)]);
        }

        builder.invoke_signed(signer_seeds)?;

        Ok(())
    }

    pub fn record_burn(&mut self) -> Result<()> {
        self.player_progress.total_cnfts_burned += 1;

        Ok(())
    }
}

pub fn handler<'info>(
    ctx: Context<'_, '_, 'info, 'info, BurncNFT<'info>>,
    root: [u8; 32],
    data_hash: [u8; 32],
    creator_hash: [u8; 32],
    nonce: u64,
    index: u32,
    asset_data_hash: [u8; 32],
    flags: u8,
) -> Result<()> {
    ctx.accounts.init_player_progress(&ctx.bumps)?;
    ctx.accounts.burn_cnft(
        root,
        data_hash,
        creator_hash,
        nonce,
        index,
        asset_data_hash,
        flags,
        ctx.remaining_accounts,
    )?;

    ctx.accounts.record_burn()?;

    Ok(())
}
