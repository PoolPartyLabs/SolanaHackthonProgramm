use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use num_bigint::BigInt;
use anchor_spl::memo::Memo;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{ Mint, Token2022, TokenAccount };
use raydium_clmm_cpi::states::PersonalPositionState;
use raydium_clmm_cpi::{
    cpi,
    program::RaydiumClmm,
    states::{ AmmConfig, ObservationState, PoolState },
    ID as RAYDIUM_CLMM_ID,
};

use crate::constants::DENOMINATOR_MULTIPLIER;
use crate::libraries::tick_math;
use crate::state::{ Investor };

#[derive(Accounts)]
pub struct SwapToRatioDepositCtx<'info> {
    #[account(address = RAYDIUM_CLMM_ID)]
    pub clmm_program: Program<'info, RaydiumClmm>,

    /// CHECK:
    #[account( mut )]
    pub pool_position_config: UncheckedAccount<'info>,

    /// The user performing the swap
    #[account(mut)]
    pub investor: Signer<'info>,

    /// The program account of the pool in which the swap will be performed
    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,

    /// Increase liquidity for this position
    #[account(mut, constraint = personal_position.pool_id == pool_state.key())]
    pub personal_position: Box<Account<'info, PersonalPositionState>>,

    /// The factory state to read protocol fees
    #[account(mut, address = pool_state_0.load()?.amm_config)]
    pub amm_config_0: Box<Account<'info, AmmConfig>>,

    /// The program account of the pool in which the swap will be performed
    #[account(mut)]
    pub pool_state_0: AccountLoader<'info, PoolState>,

    // / The factory state to read protocol fees
    #[account(mut, address = pool_state_1.load()?.amm_config)]
    pub amm_config_1: Box<Account<'info, AmmConfig>>,

    /// The program account of the pool in which the swap will be performed
    #[account(mut)]
    pub pool_state_1: AccountLoader<'info, PoolState>,

    #[account( 
      mut,
      seeds = [
          Investor::INVESTOR_SEED.as_bytes(),
          pool_position_config.key().as_ref(),
          investor.key().as_ref(),
      ],
      bump, 
  )]
    pub investor_account: Box<Account<'info, Investor>>,

    #[account(
        mut,
        seeds = [
            Investor::INVESTOR_DEPOSIT_STABLE_TOKEN_ACCOUNT_SEED.as_bytes(),
            investor_account.key().as_ref(),
        ],
        bump,
    )]
    pub investor_deposit_stable_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            Investor::INVESTOR_DEPOSIT_TOKEN_0_ACCOUNT_SEED.as_bytes(), 
            investor_account.key().as_ref(),
        ],
        bump,
    )]
    pub investor_deposit_token_0_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            Investor::INVESTOR_DEPOSIT_TOKEN_1_ACCOUNT_SEED.as_bytes(), 
            investor_account.key().as_ref(),
        ],
        bump,
    )]
    pub investor_deposit_token_1_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK:
    /// The vault token account for input token
    #[account(mut)]
    pub pool_vault_0_input: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK:
    /// The vault token account for input token
    #[account(mut)]
    pub pool_vault_1_input: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK:
    /// The vault token account for output token
    #[account(mut)]
    pub pool_vault_token_0_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// CHECK:
    /// The vault token account for output token
    #[account(mut)]
    pub pool_vault_token_1_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The mint of token vault 0
    // #[account(address = pool_vault_deposit_stable_mint.mint)]
    pub pool_vault_deposit_stable_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of token vault 1
    #[account(address = pool_vault_token_0_account.mint)]
    pub pool_vault_token_0_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of token vault 1
    #[account(address = pool_vault_token_1_account.mint)]
    pub pool_vault_token_1_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The program account for the most recent oracle observation
    #[account(mut, address = pool_state_0.load()?.observation_key)]
    pub observation_state_0: AccountLoader<'info, ObservationState>,

    /// The program account for the most recent oracle observation
    #[account(mut, address = pool_state_1.load()?.observation_key)]
    pub observation_state_1: AccountLoader<'info, ObservationState>,

    /// SPL program for token transfers
    pub token_program: Program<'info, Token>,

    /// SPL program 2022 for token transfers
    pub token_program_2022: Program<'info, Token2022>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    /// memo program
    pub memo_program: Program<'info, Memo>,

    /// System program
    pub system_program: Program<'info, System>,
}

impl<'info> SwapToRatioDepositCtx<'info> {
    pub fn swap_to_ratio_deposit<'a, 'b, 'c: 'info>(
        &mut self,
        other_amount_threshold: u64,
        sqrt_price_limit_x64: u128,
        is_base_input: bool,
        remaining_accounts: &'c [AccountInfo<'info>],
        bumps: &SwapToRatioDepositCtxBumps
    ) -> Result<()> {
        let sqrt_price_x64 = self.pool_state.load()?.sqrt_price_x64;
        let low_sqrt_price = tick_math
            ::get_sqrt_price_at_tick(self.personal_position.tick_lower_index)
            .unwrap();
        let high_sqrt_price = tick_math
            ::get_sqrt_price_at_tick(self.personal_position.tick_upper_index)
            .unwrap();

        let amount = self.investor_deposit_stable_token_account.amount;

        msg!("amount: {}", amount);

        let (amount_a, amount_b) = self.calc_ratio_amounts(
            amount,
            low_sqrt_price,
            high_sqrt_price,
            sqrt_price_x64
        )?;

        msg!("amount_a: {}", amount_a);
        msg!("amount_b: {}", amount_b);

        // Split the remaining accounts into two slices, one for each CPI
        let mut accounts_iter = remaining_accounts.iter();
        let mut split_index = 0;
        while let Some(account_info) = accounts_iter.next() {
            if account_info.key() == Pubkey::default() {
                break;
            }
            split_index += 1;
        }

        let remaining_accounts_0 = &remaining_accounts[..split_index];
        let remaining_accounts_1 = &remaining_accounts[split_index + 1..];

        let bumps = bumps.investor_deposit_stable_token_account;
        let investor_account_key = self.investor_account.key();
        let signer_seeds: &[&[&[u8]]] = &[
            &[
                Investor::INVESTOR_DEPOSIT_STABLE_TOKEN_ACCOUNT_SEED.as_bytes(),
                investor_account_key.as_ref(),
                &[bumps],
            ],
        ];
        let cpi_0_accounts = cpi::accounts::SwapSingleV2 {
            payer: self.investor_deposit_stable_token_account.to_account_info(),
            amm_config: self.amm_config_0.to_account_info(),
            pool_state: self.pool_state_0.to_account_info(),
            input_token_account: self.investor_deposit_stable_token_account.to_account_info(),
            output_token_account: self.investor_deposit_token_0_account.to_account_info(),
            input_vault: self.pool_vault_0_input.to_account_info(),
            output_vault: self.pool_vault_token_0_account.to_account_info(),
            observation_state: self.observation_state_0.to_account_info(),
            token_program: self.token_program.to_account_info(),
            token_program_2022: self.token_program_2022.to_account_info(),
            memo_program: self.memo_program.to_account_info(),
            input_vault_mint: self.pool_vault_deposit_stable_mint.to_account_info(),
            output_vault_mint: self.pool_vault_token_0_mint.to_account_info(),
        };
        let cpi_0_context = CpiContext::new(self.clmm_program.to_account_info(), cpi_0_accounts)
            .with_remaining_accounts(remaining_accounts_0.to_vec())
            .with_signer(signer_seeds);
        cpi::swap_v2(
            cpi_0_context,
            amount_a,
            other_amount_threshold,
            sqrt_price_limit_x64,
            is_base_input
        )?;

        let cpi_1_accounts = cpi::accounts::SwapSingleV2 {
            payer: self.investor_deposit_stable_token_account.to_account_info(),
            amm_config: self.amm_config_1.to_account_info(),
            pool_state: self.pool_state_1.to_account_info(),
            input_token_account: self.investor_deposit_stable_token_account.to_account_info(),
            output_token_account: self.investor_deposit_token_1_account.to_account_info(),
            input_vault: self.pool_vault_1_input.to_account_info(),
            output_vault: self.pool_vault_token_1_account.to_account_info(),
            observation_state: self.observation_state_1.to_account_info(),
            token_program: self.token_program.to_account_info(),
            token_program_2022: self.token_program_2022.to_account_info(),
            memo_program: self.memo_program.to_account_info(),
            input_vault_mint: self.pool_vault_deposit_stable_mint.to_account_info(),
            output_vault_mint: self.pool_vault_token_1_mint.to_account_info(),
        };
        let cpi_1_context = CpiContext::new(self.clmm_program.to_account_info(), cpi_1_accounts)
            .with_remaining_accounts(remaining_accounts_1.to_vec())
            .with_signer(signer_seeds);
        cpi::swap_v2(
            cpi_1_context,
            amount_b,
            other_amount_threshold,
            sqrt_price_limit_x64,
            is_base_input
        )?;
        Ok(())
    }

    fn calc_ratio_amounts(
        &self,
        amount: u64,
        low_sqrt_price: u128,
        high_sqrt_price: u128,
        current_sqrt_price: u128
    ) -> Result<(u64, u64)> {
        let denominator_multiplier = BigInt::from(DENOMINATOR_MULTIPLIER);

        let low_times_high = BigInt::from(low_sqrt_price)
            .checked_mul(&BigInt::from(high_sqrt_price))
            .unwrap();
        let high_times_current = BigInt::from(high_sqrt_price)
            .checked_mul(&BigInt::from(current_sqrt_price))
            .unwrap();

        let a = low_times_high.sqrt().checked_sub(&high_times_current.sqrt()).unwrap();
        let b = BigInt::from(current_sqrt_price).checked_sub(&high_times_current.sqrt()).unwrap();
        let c = a
            .checked_mul(&denominator_multiplier)
            .unwrap()
            .checked_div(&b)
            .unwrap()
            .checked_add(&denominator_multiplier)
            .unwrap();

        let ratio_a = denominator_multiplier
            .checked_mul(&denominator_multiplier)
            .unwrap()
            .checked_div(&c)
            .unwrap();

        let amount_1 = BigInt::from(amount)
            .checked_mul(&ratio_a)
            .unwrap()
            .checked_div(&denominator_multiplier)
            .unwrap();
        let amount_2 = BigInt::from(amount).checked_sub(&amount_1).unwrap();

        Ok((amount_1.to_string().parse().unwrap(), amount_2.to_string().parse().unwrap()))
    }
}
