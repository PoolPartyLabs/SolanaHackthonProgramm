use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token_interface::{ Mint, TokenAccount };

use crate::constants::ANCHOR_DISCRIMINATOR_SIZE;
use crate::state::{ Investor, PoolPosition };

#[derive(Accounts)]
pub struct CreateInvestorPositionCtx<'info> {
    #[account(mut)]
    pub investor: Signer<'info>,

    /// CHECK:
    #[account( mut )]
    pub pool_position_config: UncheckedAccount<'info>,

    /// CHECK:
    #[account( 
        mut,
        seeds = [
            PoolPosition::POOL_POSITION_SEED.as_bytes(),
            pool_position_config.key().as_ref(),
        ],
        bump,
    )]
    pub pool_position: Box<Account<'info, PoolPosition>>,

    #[account(
        init_if_needed,
        payer = investor,
        seeds = [
            Investor::INVESTOR_SEED.as_bytes(),
            pool_position_config.key().as_ref(),
            investor.key().as_ref(),
        ],
        bump,
        space = ANCHOR_DISCRIMINATOR_SIZE + Investor::LEN
    )]
    pub investor_account: Box<Account<'info, Investor>>,

    #[account(
        init_if_needed,
        payer = investor,
        seeds = [
            Investor::INVESTOR_DEPOSIT_TOKEN_0_ACCOUNT_SEED.as_bytes(),
            investor_account.key().as_ref(),
        ],
        bump,
        token::mint = pool_vault_token_0_mint,
        token::authority = investor_deposit_token_0_account
    )]
    pub investor_deposit_token_0_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        init_if_needed,
        payer = investor,
        seeds = [
            Investor::INVESTOR_DEPOSIT_TOKEN_1_ACCOUNT_SEED.as_bytes(),
            investor_account.key().as_ref(),
        ],
        bump,
        token::mint = pool_vault_token_1_mint,
        token::authority = investor_deposit_token_1_account
    )]
    pub investor_deposit_token_1_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The mint of token vault 0
    pub pool_vault_token_0_mint: Box<InterfaceAccount<'info, Mint>>,

    /// The mint of token vault 1
    pub pool_vault_token_1_mint: Box<InterfaceAccount<'info, Mint>>,

    /// SPL program for token transfers
    pub token_program: Program<'info, Token>,

    /// System program
    pub system_program: Program<'info, System>,
}

impl<'info> CreateInvestorPositionCtx<'info> {
    pub fn create_investor_position<'a, 'b, 'c: 'info>(
        &mut self,
        bumps: &CreateInvestorPositionCtxBumps
    ) -> Result<()> {
        let investor = &mut self.investor_account;
        investor.bump = bumps.investor_account;
        investor.authority = *self.investor.key;
        investor.pool_position_key = self.pool_position.key();

        Ok(())
    }
}
