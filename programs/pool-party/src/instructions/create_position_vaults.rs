use anchor_lang::prelude::*;
use anchor_spl::{ token::Token, token_interface::{ Mint, TokenAccount } };

use crate::{ errors, state::PoolPosition };

#[derive(Accounts)]
pub struct CreatePositionVaultsCtx<'info> {
    #[account(mut)]
    pub manager: Signer<'info>,

    /// CHECK:
    #[account( mut )]
    pub pool_position: UncheckedAccount<'info>,

    #[account(
        init,
        payer = manager,
        seeds = [PoolPosition::POOL_POSITION_VAULT_0_SEED.as_bytes(), pool_position.key().as_ref()],
        bump,
        token::mint = vault_0_mint,
        // Must be the pool position account because the NFT owner in the raydium is the pool position account
        token::authority = pool_position
    )]
    pub pool_position_vault_0_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = manager,
        seeds = [PoolPosition::POOL_POSITION_VAULT_1_SEED.as_bytes(), pool_position.key().as_ref()],
        bump,
        token::mint = vault_1_mint,
        // Must be the pool position account because the NFT owner in the raydium is the pool position account
        token::authority = pool_position
    )]
    pub pool_position_vault_1_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = manager,
        seeds = [
            PoolPosition::POOL_POSITION_FEES_VAULT_0_SEED.as_bytes(),
            pool_position.key().as_ref(),
        ],
        bump,
        token::mint = vault_0_mint,
        token::authority = fees_vault_0_token_account
    )]
    pub fees_vault_0_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init,
        payer = manager,
        seeds = [
            PoolPosition::POOL_POSITION_FEES_VAULT_1_SEED.as_bytes(),
            pool_position.key().as_ref(),
        ],
        bump,
        token::mint = vault_1_mint,
        token::authority = fees_vault_1_token_account
    )]
    pub fees_vault_1_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The mint of token vault 0
    pub vault_0_mint: Box<InterfaceAccount<'info, Mint>>,
    /// The mint of token vault 1
    pub vault_1_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,

    /// Program to create the position manager state account
    pub system_program: Program<'info, System>,
}

impl<'info> CreatePositionVaultsCtx<'info> {
    pub fn create_position_vaults(&mut self) -> Result<()> {
        let mut data: &[u8] = &mut self.pool_position.try_borrow_data()?;
        let mut pool_position = PoolPosition::try_deserialize(&mut data)?;
        if pool_position.vaults_initialized {
            return Err(errors::ErrorCode::VaultsAlreadyInitialized.into());
        }
        pool_position.vaults_initialized = true;
        let mut vec_data = pool_position.try_to_vec()?;
        let slice_data: &mut [u8] = &mut vec_data[..];

        let result = self.pool_position.serialize_data(&slice_data);
        if result.is_err() {
            return Err(anchor_lang::error::ErrorCode::AccountDidNotSerialize.into());
        }
        Ok(())
    }
}
