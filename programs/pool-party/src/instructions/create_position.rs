use anchor_lang::prelude::*;
use anchor_spl::token::Token;

use crate::{
    constants::ANCHOR_DISCRIMINATOR_SIZE,
    state::{ Investor, PoolPosition, PoolPositionConfig },
};

#[derive(Accounts)]
#[instruction(
    name: String,
)]
pub struct CreatePositionCtx<'info> {
    #[account(mut)]
    pub manager: Signer<'info>,

    #[account(
        init,
        payer = manager,
        seeds = [PoolPositionConfig::POOL_POSITION_CONFIG_SEED.as_bytes(), name.as_bytes()],
        bump,
        space = ANCHOR_DISCRIMINATOR_SIZE + PoolPositionConfig::LEN + 4 + name.len()
    )]
    pub pool_position_config: Box<Account<'info, PoolPositionConfig>>,

    #[account(
        init,
        payer = manager,
        seeds = [PoolPosition::POOL_POSITION_SEED.as_bytes(), pool_position_config.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR_SIZE + PoolPosition::LEN
    )]
    pub pool_position: Box<Account<'info, PoolPosition>>,

    #[account(
        init,
        payer = manager,
        seeds = [
            Investor::INVESTOR_SEED.as_bytes(),
            pool_position_config.key().as_ref(),
            manager.key().as_ref(),
        ],
        bump,
        space = ANCHOR_DISCRIMINATOR_SIZE + Investor::LEN
    )]
    pub manager_account: Box<Account<'info, Investor>>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,

    /// Program to create the position manager state account
    pub system_program: Program<'info, System>,
}

impl<'info> CreatePositionCtx<'info> {
    pub fn create_position(
        &mut self,
        name: String,
        tick_lower_index: i32,
        tick_upper_index: i32,
        pool_state_key: Pubkey,
        token_vault_0_key: Pubkey,
        token_vault_1_key: Pubkey,
        vault_0_mint_key: Pubkey,
        vault_1_mint_key: Pubkey,
        bumps: &CreatePositionCtxBumps
    ) -> Result<()> {
        let pool_position_config = &mut self.pool_position_config;
        let pool_position = &mut self.pool_position;
        pool_position_config.name = name.clone();
        pool_position_config.tick_lower_index = tick_lower_index;
        pool_position_config.tick_upper_index = tick_upper_index;
        pool_position_config.pool_key = pool_state_key;
        pool_position_config.pool_position_key = pool_position.key();
        pool_position_config.manager_key = self.manager.key();
        pool_position_config.token_vault_0_key = token_vault_0_key;
        pool_position_config.token_vault_1_key = token_vault_1_key;
        pool_position_config.vault_0_mint_key = vault_0_mint_key;
        pool_position_config.vault_1_mint_key = vault_1_mint_key;
        pool_position_config.bump = bumps.pool_position_config;

        let manager_account = &mut self.manager_account;

        pool_position.bump = bumps.pool_position;
        pool_position.pool_position_config_key = pool_position_config.key();
        pool_position.liquidity = 0;

        manager_account.bump = bumps.manager_account;
        manager_account.authority = self.manager.key();
        manager_account.pool_position_key = pool_position.key();
        manager_account.is_manager = true;

        Ok(())
    }
}
