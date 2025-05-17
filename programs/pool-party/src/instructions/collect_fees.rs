use anchor_lang::prelude::*;
use anchor_lang::system_program::System;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::memo::spl_memo;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{ Mint, Token2022, TokenAccount };
use num_bigint::BigInt;
use raydium_clmm_cpi::{
    cpi,
    program::RaydiumClmm,
    states::{ PersonalPositionState, PoolState, ProtocolPositionState, TickArrayState },
    ID as RAYDIUM_CLMM_ID,
};
use crate::libraries::{ transfer_token, fixed_point_64, MulDiv, U128 };
use crate::state::{ tick_array, Investor, PoolPosition, PoolPositionConfig, TickArrayStateExt };

pub struct CollectFeesArgs<'info> {
    pub clmm_program: AccountInfo<'info>,
    pub nft_owner: AccountInfo<'info>,
    pub nft_account: AccountInfo<'info>,
    pub pool_state: AccountInfo<'info>,
    pub protocol_position: AccountInfo<'info>,
    pub personal_position: AccountInfo<'info>,
    pub tick_array_lower: AccountInfo<'info>,
    pub tick_array_upper: AccountInfo<'info>,
    pub recipient_token_account_0: AccountInfo<'info>,
    pub recipient_token_account_1: AccountInfo<'info>,
    pub token_vault_0: AccountInfo<'info>,
    pub token_vault_1: AccountInfo<'info>,
    pub token_program: AccountInfo<'info>,
    pub token_program_2022: AccountInfo<'info>,
    pub vault_0_mint: AccountInfo<'info>,
    pub vault_1_mint: AccountInfo<'info>,
    pub memo_program: AccountInfo<'info>,
    pub remaining_accounts: Vec<AccountInfo<'info>>,
}

#[derive(Accounts)]
pub struct CollectFeesCtx<'info> {
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

    #[account(
        init_if_needed,
        payer = investor,
        associated_token::mint = pool_vault_token_a_mint,
        associated_token::authority = investor,
        associated_token::token_program = token_program
    )]
    pub investor_deposit_fees_0_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = investor,
        associated_token::mint = pool_vault_token_b_mint,
        associated_token::authority = investor,
        associated_token::token_program = token_program
    )]
    pub investor_deposit_fees_1_account: Box<InterfaceAccount<'info, TokenAccount>>,

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

    /// Required for init_if_needed constraint
    pub system_program: Program<'info, System>,

    // remaining account
    // #[account(
    //     seeds = [
    //         POOL_TICK_ARRAY_BITMAP_SEED.as_bytes(),
    //         pool_state.key().as_ref(),
    //     ],
    //     bump
    // )]
    // pub tick_array_bitmap: AccountLoader<'info, TickArrayBitmapExtension>,
    // pub tick_array_bitmap: AccountLoader<'info, TickArrayBitmapExtension>,
}

impl<'info> CollectFeesCtx<'info> {
    pub fn collect_fees<'a, 'b, 'c: 'info>(
        &mut self,
        remaining_accounts: &'c [AccountInfo<'info>],
        bumps: &CollectFeesCtxBumps
    ) -> Result<()> {
        let (fees_owed0, fees_owed1) = get_owed_fees(
            self.tick_array_lower.clone(),
            self.tick_array_upper.clone(),
            &self.personal_position,
            &self.pool_state
        );
        msg!("fees_owed0: {}", fees_owed0);
        msg!("fees_owed1: {}", fees_owed1);

        if fees_owed0 > 0 || fees_owed1 > 0 {
            let pool_position_bump_seed = self.pool_position.bump;
            let pool_position_config_key = self.pool_position_config.key();
            let signer_seeds: &[&[&[u8]]] = &[
                &[
                    PoolPosition::POOL_POSITION_SEED.as_bytes(),
                    pool_position_config_key.as_ref(),
                    &[pool_position_bump_seed],
                ],
            ];

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
        let investor_account = self.investor_account.clone();
        let pool_position = self.pool_position.clone();
        let liquidity = self.personal_position.liquidity;

        let (fees_index0, fees_index1) = fees_indexes(liquidity, fees_owed0, fees_owed1);

        let last_fees_index0 = pool_position.fees_index0;
        let last_fees_index1 = pool_position.fees_index1;

        let investor_liquidity = investor_account.liquidity;

        let fees_index0 = U128::from(last_fees_index0)
            .checked_add(U128::from(fees_index0))
            .unwrap()
            .as_u128();
        let fees_index1 = U128::from(last_fees_index1)
            .checked_add(U128::from(fees_index1))
            .unwrap()
            .as_u128();

        let fees_earned0 = investor_account.fees_earned0
            .checked_add(
                calculate_fees(investor_liquidity, fees_index0, investor_account.fees_index0)
            )
            .unwrap();
        let fees_earned1 = investor_account.fees_earned1
            .checked_add(
                calculate_fees(investor_liquidity, fees_index1, investor_account.fees_index1)
            )
            .unwrap();

        self.transfer_fees(fees_earned0, fees_earned1, bumps)?;

        let pool_position = &mut self.pool_position;
        let investor_account = &mut self.investor_account;

        pool_position.fees_index0 = fees_index0;
        pool_position.fees_index1 = fees_index1;
        investor_account.fees_earned0 = 0;
        investor_account.fees_earned1 = 0;
        investor_account.fees_index0 = pool_position.fees_index0;
        investor_account.fees_index1 = pool_position.fees_index1;

        Ok(())
    }

    fn transfer_fees(
        &self,
        amount_0: u64,
        amount_1: u64,
        bumps: &CollectFeesCtxBumps
    ) -> Result<()> {
        if amount_0 == 0 && amount_1 == 0 {
            msg!("No fees to transfer");
            return Ok(());
        }
        let pool_position_key = self.pool_position.key();
        let fees_vault_0_token_account_bump_seed = bumps.fees_vault_0_token_account;
        let a_seeds: &[&[&[u8]]] = &[
            &[
                PoolPosition::POOL_POSITION_FEES_VAULT_0_SEED.as_bytes(),
                pool_position_key.as_ref(),
                &[fees_vault_0_token_account_bump_seed],
            ],
        ];
        let token_program = self.token_program.clone();
        let recipient_0 = self.investor_deposit_fees_0_account.clone();
        let fees_vault_0_token_account = self.fees_vault_0_token_account.clone();
        let token_0_mint_acc = self.pool_vault_token_a_mint.clone();
        transfer_token(
            &fees_vault_0_token_account,
            &recipient_0,
            &amount_0,
            &token_0_mint_acc,
            &fees_vault_0_token_account.to_account_info(),
            &token_program,
            Some(a_seeds)
        )?;

        let fees_vault_1_token_account_bump_seed = bumps.fees_vault_1_token_account;
        let b_seeds: &[&[&[u8]]] = &[
            &[
                PoolPosition::POOL_POSITION_FEES_VAULT_1_SEED.as_bytes(),
                pool_position_key.as_ref(),
                &[fees_vault_1_token_account_bump_seed],
            ],
        ];
        let recipient_1 = self.investor_deposit_fees_1_account.clone();
        let fees_vault_1_token_account = self.fees_vault_1_token_account.clone();
        let token_1_mint_acc = self.pool_vault_token_b_mint.clone();
        transfer_token(
            &fees_vault_1_token_account,
            &recipient_1,
            &amount_1,
            &token_1_mint_acc,
            &fees_vault_1_token_account.to_account_info(),
            &token_program,
            Some(b_seeds)
        )
    }
}

pub fn get_owed_fees<'info>(
    tick_array_lower: AccountLoader<'info, TickArrayState>,
    tick_array_upper: AccountLoader<'info, TickArrayState>,
    personal_position: &PersonalPositionState,
    pool_state: &AccountLoader<'info, PoolState>
) -> (u64, u64) {
    let pool_state = match pool_state.clone().load() {
        Ok(state) => Box::new(state.clone()),
        Err(_) => {
            msg!("Failed to load pool_state");
            return (0, 0);
        }
    };
    let (fee_growth_inside_0_last_x64, fee_growth_inside_1_last_x64) = get_fees_growth_inside_last(
        tick_array_lower.clone(),
        tick_array_upper.clone(),
        &personal_position,
        &pool_state
    );

    let fees_owed0 = calculate_latest_fees(
        personal_position.token_fees_owed_0,
        personal_position.fee_growth_inside_0_last_x64,
        fee_growth_inside_0_last_x64,
        personal_position.liquidity
    );
    let fees_owed1 = calculate_latest_fees(
        personal_position.token_fees_owed_1,
        personal_position.fee_growth_inside_1_last_x64,
        fee_growth_inside_1_last_x64,
        personal_position.liquidity
    );

    (fees_owed0, fees_owed1)
}

pub fn get_fees_growth_inside_last<'info>(
    tick_array_lower: AccountLoader<'info, TickArrayState>,
    tick_array_upper: AccountLoader<'info, TickArrayState>,
    personal_position: &PersonalPositionState,
    pool_state: &PoolState
) -> (u128, u128) {
    let tick_array_lower_loader = tick_array_lower.load().unwrap();
    let tick_array_upper_loader = tick_array_upper.load().unwrap();

    let tick_lower_index = personal_position.tick_lower_index;
    let tick_upper_index = personal_position.tick_upper_index;

    let tick_lower_state = tick_array_lower_loader
        .get_tick_state(tick_lower_index, pool_state.tick_spacing)
        .unwrap();
    let tick_upper_state = tick_array_upper_loader
        .get_tick_state(tick_upper_index, pool_state.tick_spacing)
        .unwrap();

    let (fee_growth_inside_0_last_x64, fee_growth_inside_1_last_x64) =
        tick_array::get_fee_growth_inside(
            &tick_lower_state,
            &tick_upper_state,
            pool_state.tick_current,
            pool_state.fee_growth_global_0_x64,
            pool_state.fee_growth_global_1_x64
        );

    (fee_growth_inside_0_last_x64, fee_growth_inside_1_last_x64)
}

pub fn calculate_latest_fees(
    last_total_fees: u64,
    fee_growth_inside_last_x64: u128,
    fee_growth_inside_latest_x64: u128,
    liquidity: u128
) -> u64 {
    if fee_growth_inside_latest_x64 <= fee_growth_inside_last_x64 || liquidity == 0 {
        return last_total_fees;
    }
    let fee_growth_delta = U128::from(
        fee_growth_inside_latest_x64.wrapping_sub(fee_growth_inside_last_x64)
    )
        .mul_div_floor(U128::from(liquidity), U128::from(fixed_point_64::Q64))
        .unwrap()
        .to_underflow_u64();

    last_total_fees.checked_add(fee_growth_delta).unwrap()
}

pub fn fees_indexes(liquidity: u128, fees_owed0: u64, fees_owed1: u64) -> (u128, u128) {
    let denominator_multiplier = BigInt::from(10).pow(10);
    let mut fees_index_0: u128 = 0;
    let mut fees_index_1: u128 = 0;

    if fees_owed0 > 0 && liquidity > 0 {
        fees_index_0 = BigInt::from(fees_owed0)
            .checked_mul(&denominator_multiplier)
            .unwrap()
            .checked_div(&BigInt::from(liquidity))
            .unwrap()
            .to_string()
            .parse::<u128>()
            .unwrap();
    }
    if fees_owed1 > 0 && liquidity > 0 {
        fees_index_1 = BigInt::from(fees_owed1)
            .checked_mul(&denominator_multiplier)
            .unwrap()
            .checked_div(&BigInt::from(liquidity))
            .unwrap()
            .to_string()
            .parse::<u128>()
            .unwrap();
    }

    (fees_index_0, fees_index_1)
}

pub fn calculate_fees(investor_liquidity: u128, fee_index: u128, investor_fee_index: u128) -> u64 {
    let denominator_multiplier = BigInt::from(10).pow(10);
    let res = BigInt::from(investor_liquidity)
        .checked_mul(
            &BigInt::from(fee_index).checked_sub(&BigInt::from(investor_fee_index)).unwrap()
        )
        .unwrap()
        .checked_div(&denominator_multiplier)
        .unwrap()
        .to_string();
    res.parse::<u64>().unwrap()
}

pub fn collect_fees<'info>(args: CollectFeesArgs<'info>, signer_seeds: &[&[&[u8]]]) -> Result<()> {
    let cpi_accounts = cpi::accounts::DecreaseLiquidityV2 {
        nft_owner: args.nft_owner,
        nft_account: args.nft_account,
        pool_state: args.pool_state,
        protocol_position: args.protocol_position,
        personal_position: args.personal_position,
        tick_array_lower: args.tick_array_lower,
        tick_array_upper: args.tick_array_upper,
        recipient_token_account_0: args.recipient_token_account_0,
        recipient_token_account_1: args.recipient_token_account_1,
        token_vault_0: args.token_vault_0,
        token_vault_1: args.token_vault_1,
        token_program: args.token_program,
        token_program_2022: args.token_program_2022,
        vault_0_mint: args.vault_0_mint,
        vault_1_mint: args.vault_1_mint,
        memo_program: args.memo_program,
    };

    let cpi_context = CpiContext::new_with_signer(
        args.clmm_program.to_account_info(),
        cpi_accounts,
        signer_seeds
    ).with_remaining_accounts(args.remaining_accounts.to_vec());
    cpi::decrease_liquidity_v2(cpi_context, 0, 0, 0)
}
