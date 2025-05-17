use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;
pub mod libraries;
use core as core_;

#[allow(unused_imports)]
use instructions::*;
#[allow(unused_imports)]
use constants::*;
#[allow(unused_imports)]
use errors::*;
#[allow(unused_imports)]
use state::*;

declare_id!("3inmw7qcywQirQoNSL54MhqoG7CJ58ZYwVCYSmC1TTB4");

#[program]
pub mod pool_party {
    use super::*;

    pub fn create_position<'info>(
        ctx: Context<CreatePositionCtx<'info>>,
        name: String,
        tick_lower_index: i32,
        tick_upper_index: i32,
        pool_state_key: Pubkey,
        token_vault_0_key: Pubkey,
        token_vault_1_key: Pubkey,
        vault_0_mint_key: Pubkey,
        vault_1_mint_key: Pubkey
    ) -> Result<()> {
        ctx.accounts.create_position(
            name,
            tick_lower_index,
            tick_upper_index,
            pool_state_key,
            token_vault_0_key,
            token_vault_1_key,
            vault_0_mint_key,
            vault_1_mint_key,
            &ctx.bumps
        )
    }

    pub fn create_position_vaults<'info>(
        _ctx: Context<CreatePositionVaultsCtx<'info>>
    ) -> Result<()> {
        Ok(())
    }

    pub fn open_position<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, OpenPositionCtx<'info>>,
        amount_0_max: u64,
        amount_1_max: u64,
        tick_lower_index: i32,
        tick_upper_index: i32,
        tick_array_lower_start_index: i32,
        tick_array_upper_start_index: i32
    ) -> Result<()> {
        ctx.accounts.open_position(
            amount_0_max,
            amount_1_max,
            tick_lower_index,
            tick_upper_index,
            tick_array_lower_start_index,
            tick_array_upper_start_index,
            ctx.remaining_accounts
        )
    }

    pub fn create_investor_position<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CreateInvestorPositionCtx<'info>>
    ) -> Result<()> {
        ctx.accounts.create_investor_position(&ctx.bumps)
    }

    pub fn deposit<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, DepositCtx<'info>>,
        amount: u64
    ) -> Result<()> {
        ctx.accounts.deposit(amount)
    }

    pub fn swap_to_ratio_deposit<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, SwapToRatioDepositCtx<'info>>,
        other_amount_threshold: u64,
        sqrt_price_limit_x64: u128,
        is_base_input: bool
    ) -> Result<()> {
        ctx.accounts.swap_to_ratio_deposit(
            other_amount_threshold,
            sqrt_price_limit_x64,
            is_base_input,
            ctx.remaining_accounts,
            &ctx.bumps
        )
    }

    pub fn increase_liquidity<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, IncreaseLiquidityCtx<'info>>
    ) -> Result<()> {
        ctx.accounts.increase_liquidity(ctx.remaining_accounts, &ctx.bumps)
    }

    pub fn collect_fees<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, CollectFeesCtx<'info>>
    ) -> Result<()> {
        ctx.accounts.collect_fees(ctx.remaining_accounts, &ctx.bumps)
    }
}
