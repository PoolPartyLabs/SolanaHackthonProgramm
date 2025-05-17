use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::memo::spl_memo;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{ Mint, Token2022, TokenAccount };
use raydium_clmm_cpi::{
    cpi,
    program::RaydiumClmm,
    states::{ PersonalPositionState, PoolState, ProtocolPositionState, TickArrayState },
    ID as RAYDIUM_CLMM_ID,
};

use crate::instructions::{
    calculate_fees,
    collect_fees,
    fees_indexes,
    get_owed_fees,
    updated_liquidity_personal_position,
    CollectFeesArgs,
};
use crate::libraries::{ transfer_token, U128 };
use crate::state::{ Investor, PoolPosition, PoolPositionConfig };

#[derive(Accounts)]
pub struct IncreaseLiquidityCtx<'info> {
    #[account(mut)]
    pub investor: Signer<'info>,

    /// CHECK:
    #[account()]
    pub pool_position_config: Box<Account<'info, PoolPositionConfig>>,

    /// CHECK:
    #[account(
        mut,
        seeds = [PoolPosition::POOL_POSITION_SEED.as_bytes(), pool_position_config.key().as_ref()],
        bump,
    )]
    pub pool_position: Box<Account<'info, PoolPosition>>,

    /// CHECK:
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
                Investor::INVESTOR_DEPOSIT_TOKEN_0_ACCOUNT_SEED.as_bytes(), 
                investor_account.key().as_ref(),
            ],
            bump,
      )]
    pub investor_deposit_token_a_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            Investor::INVESTOR_DEPOSIT_TOKEN_1_ACCOUNT_SEED.as_bytes(), 
            investor_account.key().as_ref(),
        ],
        bump,
    )]
    pub investor_deposit_token_b_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [PoolPosition::POOL_POSITION_VAULT_0_SEED.as_bytes(), pool_position.key().as_ref()],
        bump,
    )]
    pub pool_position_vault_0_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [PoolPosition::POOL_POSITION_VAULT_1_SEED.as_bytes(), pool_position.key().as_ref()],
        bump,
    )]
    pub pool_position_vault_1_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,

        seeds = [
            PoolPosition::POOL_POSITION_FEES_VAULT_0_SEED.as_bytes(),
            pool_position.key().as_ref(),
        ],
        bump,
    )]
    pub fees_vault_0_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            PoolPosition::POOL_POSITION_FEES_VAULT_1_SEED.as_bytes(),
            pool_position.key().as_ref(),
        ],
        bump,
    )]
    pub fees_vault_1_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(address = RAYDIUM_CLMM_ID)]
    pub clmm_program: Program<'info, RaydiumClmm>,

    /// The token account for nft
    #[account(
        constraint = position_nft_account.mint == personal_position.nft_mint,
        token::token_program = token_program
    )]
    pub position_nft_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,

    #[account(
        mut,
        constraint = protocol_position.pool_id == pool_state.key(),
    )]
    pub protocol_position: Box<Account<'info, ProtocolPositionState>>,

    /// Increase liquidity for this position
    #[account(mut, constraint = personal_position.pool_id == pool_state.key())]
    pub personal_position: Box<Account<'info, PersonalPositionState>>,

    /// Stores init state for the lower tick
    #[account(mut, constraint = tick_array_lower.load()?.pool_id == pool_state.key())]
    pub tick_array_lower: AccountLoader<'info, TickArrayState>,

    /// Stores init state for the upper tick
    #[account(mut, constraint = tick_array_upper.load()?.pool_id == pool_state.key())]
    pub tick_array_upper: AccountLoader<'info, TickArrayState>,

    /// The address that holds pool tokens for token_0
    #[account(
        mut,
        constraint = token_vault_0.key() == pool_state.load()?.token_vault_0
    )]
    pub token_vault_0: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that holds pool tokens for token_1
    #[account(
        mut,
        constraint = token_vault_1.key() == pool_state.load()?.token_vault_1
    )]
    pub token_vault_1: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The mint of token vault 0
    #[account(address = token_vault_0.mint)]
    pub pool_vault_token_a_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of token vault 1
    #[account(address = token_vault_1.mint)]
    pub pool_vault_token_b_mint: Box<InterfaceAccount<'info, Mint>>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,

    /// Token program 2022
    pub token_program_2022: Program<'info, Token2022>,

    /// memo program
    /// CHECK:
    #[account(address = spl_memo::id())]
    pub memo_program: UncheckedAccount<'info>,

    // remaining account
    // #[account(
    //     seeds = [
    //         POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
    //         pool_state.key().as_ref(),
    //     ],
    //     bump
    // )]
    // pub tick_array_bitmap: AccountLoader<'info, TickArrayBitmapExtension>,
}

impl<'info> IncreaseLiquidityCtx<'info> {
    pub fn increase_liquidity<'a, 'b, 'c: 'info>(
        &mut self,
        remaining_accounts: &'c [AccountInfo<'info>],
        bumps: &IncreaseLiquidityCtxBumps
    ) -> Result<()> {
        let amount_0_max = self.investor_deposit_token_a_account.amount;
        let amount_1_max = self.investor_deposit_token_b_account.amount;

        msg!("vaults_initialized: {}", self.pool_position.vaults_initialized);

        self.transfer_tokens(amount_0_max, amount_1_max, bumps)?;

        let (fees_owed0, fees_owed1) = get_owed_fees(
            self.tick_array_lower.clone(),
            self.tick_array_upper.clone(),
            &self.personal_position,
            &self.pool_state
        );

        let personal_position = self.personal_position.clone();
        let pool_position = self.pool_position.clone();
        let investor_account = &mut self.investor_account;
        let liquidity = personal_position.liquidity;

        let (feess_index0, feess_index1) = fees_indexes(liquidity, fees_owed0, fees_owed1);

        let last_fees_index0 = pool_position.fees_index0;
        let last_fees_index1 = pool_position.fees_index1;

        let investor_liquidity = investor_account.liquidity;

        let fees_index0 = U128::from(last_fees_index0)
            .checked_add(U128::from(feess_index0))
            .unwrap()
            .as_u128();
        let fees_index1 = U128::from(last_fees_index1)
            .checked_add(U128::from(feess_index1))
            .unwrap()
            .as_u128();

        investor_account.fees_earned0 = investor_account.fees_earned0
            .checked_add(
                calculate_fees(
                    investor_liquidity,
                    pool_position.fees_index0,
                    investor_account.fees_index0
                )
            )
            .unwrap();
        investor_account.fees_earned1 = investor_account.fees_earned1
            .checked_add(
                calculate_fees(
                    investor_liquidity,
                    pool_position.fees_index1,
                    investor_account.fees_index1
                )
            )
            .unwrap();
        investor_account.fees_index0 = pool_position.fees_index0;
        investor_account.fees_index1 = pool_position.fees_index1;

        let pool_position_bump_seed = self.pool_position.bump;
        let pool_position_config_key = self.pool_position_config.key();
        let signer_seeds: &[&[&[u8]]] = &[
            &[
                PoolPosition::POOL_POSITION_SEED.as_bytes(),
                pool_position_config_key.as_ref(),
                &[pool_position_bump_seed],
            ],
        ];

        if fees_owed0 > 0 || fees_owed1 > 0 {
            collect_fees(
                CollectFeesArgs {
                    clmm_program: self.clmm_program.to_account_info(),
                    nft_owner: self.pool_position.to_account_info(),
                    nft_account: self.position_nft_account.to_account_info(),
                    pool_state: self.pool_state.to_account_info(),
                    protocol_position: self.protocol_position.to_account_info(),
                    personal_position: self.personal_position.to_account_info(),
                    tick_array_lower: self.tick_array_lower.to_account_info(),
                    tick_array_upper: self.tick_array_upper.to_account_info(),
                    recipient_token_account_0: self.fees_vault_0_token_account.to_account_info(),
                    recipient_token_account_1: self.fees_vault_1_token_account.to_account_info(),
                    token_vault_0: self.token_vault_0.to_account_info(),
                    token_vault_1: self.token_vault_1.to_account_info(),
                    token_program: self.token_program.to_account_info(),
                    token_program_2022: self.token_program_2022.to_account_info(),
                    vault_0_mint: self.pool_vault_token_a_mint.to_account_info(),
                    vault_1_mint: self.pool_vault_token_b_mint.to_account_info(),
                    memo_program: self.memo_program.to_account_info(),
                    remaining_accounts: remaining_accounts.to_vec(),
                },
                signer_seeds
            )?;
        }

        let cpi_accounts = cpi::accounts::IncreaseLiquidityV2 {
            nft_owner: self.pool_position.to_account_info(),
            nft_account: self.position_nft_account.to_account_info(),
            pool_state: self.pool_state.to_account_info(),
            protocol_position: self.protocol_position.to_account_info(),
            personal_position: self.personal_position.to_account_info(),
            tick_array_lower: self.tick_array_lower.to_account_info(),
            tick_array_upper: self.tick_array_upper.to_account_info(),
            token_account_0: self.pool_position_vault_0_token_account.to_account_info(),
            token_account_1: self.pool_position_vault_1_token_account.to_account_info(),
            token_vault_0: self.token_vault_0.to_account_info(),
            token_vault_1: self.token_vault_1.to_account_info(),
            token_program: self.token_program.to_account_info(),
            token_program_2022: self.token_program_2022.to_account_info(),
            vault_0_mint: self.pool_vault_token_a_mint.to_account_info(),
            vault_1_mint: self.pool_vault_token_b_mint.to_account_info(),
        };
        let cpi_context = CpiContext::new_with_signer(
            self.clmm_program.to_account_info(),
            cpi_accounts,
            signer_seeds
        ).with_remaining_accounts(remaining_accounts.to_vec());
        cpi::increase_liquidity_v2(cpi_context, 0, amount_0_max, amount_1_max, Some(true))?;

        let liquidity_after = updated_liquidity_personal_position(
            self.personal_position.to_account_info()
        )?;

        let liquidity_delta = liquidity_after.checked_sub(liquidity).unwrap();
        investor_account.liquidity = investor_account.liquidity
            .checked_add(liquidity_delta)
            .unwrap();

        let pool_position = &mut self.pool_position;
        pool_position.liquidity = liquidity_after;
        pool_position.fees_index0 = fees_index0;
        pool_position.fees_index1 = fees_index1;

        Ok(())
    }

    pub fn transfer_tokens(
        &self,
        amount_0_max: u64,
        amount_1_max: u64,
        bumps: &IncreaseLiquidityCtxBumps
    ) -> Result<()> {
        let investor_account_key = self.investor_account.key();
        let investor_deposit_token_a_account_bump_seed = bumps.investor_deposit_token_a_account;
        let a_seeds: &[&[&[u8]]] = &[
            &[
                Investor::INVESTOR_DEPOSIT_TOKEN_0_ACCOUNT_SEED.as_bytes(),
                investor_account_key.as_ref(),
                &[investor_deposit_token_a_account_bump_seed],
            ],
        ];
        let token_program = self.token_program.clone();
        let vault_0 = self.pool_position_vault_0_token_account.clone();
        let investor_deposit_token_a_account = self.investor_deposit_token_a_account.clone();
        let token_0_mint_acc = self.pool_vault_token_a_mint.clone();
        transfer_token(
            &investor_deposit_token_a_account,
            &vault_0,
            &amount_0_max,
            &token_0_mint_acc,
            &investor_deposit_token_a_account.to_account_info(),
            &token_program,
            Some(a_seeds)
        )?;
        let investor_deposit_token_b_account_bump_seed = bumps.investor_deposit_token_b_account;
        let b_seeds: &[&[&[u8]]] = &[
            &[
                Investor::INVESTOR_DEPOSIT_TOKEN_1_ACCOUNT_SEED.as_bytes(),
                investor_account_key.as_ref(),
                &[investor_deposit_token_b_account_bump_seed],
            ],
        ];
        let vault_1 = self.pool_position_vault_1_token_account.clone();
        let investor_deposit_token_b_account = self.investor_deposit_token_b_account.clone();
        let token_1_mint_acc = self.pool_vault_token_b_mint.clone();
        transfer_token(
            &investor_deposit_token_b_account,
            &vault_1,
            &amount_1_max,
            &token_1_mint_acc,
            &investor_deposit_token_b_account.to_account_info(),
            &token_program,
            Some(b_seeds)
        )
    }
}
