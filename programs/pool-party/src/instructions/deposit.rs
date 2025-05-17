use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{ Token };
use anchor_spl::token_interface::{ Mint, TokenAccount };

use crate::libraries::{ transfer_sol };
use crate::state::{ Investor };

#[derive(Accounts)]
pub struct DepositCtx<'info> {
    #[account(mut)]
    pub investor: Signer<'info>,

    /// CHECK:
    #[account( mut )]
    pub pool_position_config: UncheckedAccount<'info>,

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
        init_if_needed,
        payer = investor,
        seeds = [
            Investor::INVESTOR_DEPOSIT_STABLE_TOKEN_ACCOUNT_SEED.as_bytes(),
            investor_account.key().as_ref(),
        ],
        bump,
        token::mint = pool_vault_deposit_stable_mint,
        token::authority = investor_deposit_stable_token_account
    )]
    pub investor_deposit_stable_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The mint of token vault stable
    #[account(mint::token_program = token_program)]
    pub pool_vault_deposit_stable_mint: Box<InterfaceAccount<'info, Mint>>,

    /// SPL program for token transfers
    pub token_program: Program<'info, Token>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    /// System program
    pub system_program: Program<'info, System>,
}

impl<'info> DepositCtx<'info> {
    pub fn deposit<'a, 'b, 'c: 'info>(&mut self, amount: u64) -> Result<()> {
        transfer_sol(
            &self.investor.to_account_info(),
            &self.investor_deposit_stable_token_account.to_account_info(),
            &self.token_program.clone(),
            &self.system_program,
            amount
        )
    }
}
