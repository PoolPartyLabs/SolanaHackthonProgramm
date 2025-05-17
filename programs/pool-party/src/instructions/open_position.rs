use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::Metadata,
    token::Token,
    token_interface::{ Mint, Token2022, TokenAccount },
};

use raydium_clmm_cpi::{
    cpi,
    program::RaydiumClmm,
    states::{ PersonalPositionState, PoolState, ProtocolPositionState },
    ID as RAYDIUM_CLMM_ID,
};

use crate::state::{ Investor, PoolPosition, PoolPositionConfig };

#[derive(Accounts)]
pub struct OpenPositionCtx<'info> {
    #[account(mut)]
    pub manager: Signer<'info>,

    #[account( mut )]
    pub pool_position_config: Box<Account<'info, PoolPositionConfig>>,

    #[account(
        mut, 
        seeds = [PoolPosition::POOL_POSITION_SEED.as_bytes(), pool_position_config.key().as_ref()],
        bump,
    )]
    pub pool_position: Box<Account<'info, PoolPosition>>,

    #[account(
        mut,
        seeds = [
            Investor::INVESTOR_SEED.as_bytes(),
            pool_position_config.key().as_ref(),
            manager.key().as_ref(),
        ],
        bump,
    )]
    pub manager_account: Box<Account<'info, Investor>>,

    #[account(address = RAYDIUM_CLMM_ID)]
    pub clmm_program: Program<'info, RaydiumClmm>,

    /// CHECK: Unique token mint address, random keypair
    #[account(mut)]
    pub position_nft_mint: Signer<'info>,

    /// CHECK: Token account where position NFT will be minted
    /// This account created in the contract by cpi to avoid large stack variables
    #[account(mut)]
    pub position_nft_account: UncheckedAccount<'info>,

    // To store metaplex metadata
    /// CHECK: Safety check performed inside function body
    #[account(mut)]
    pub metadata_account: UncheckedAccount<'info>,

    /// CHECK: Add liquidity for this pool
    #[account(mut)]
    pub pool_state: AccountLoader<'info, PoolState>,

    /// CHECK: Store the information of market marking in range
    #[account(
        mut, 
    )]
    pub protocol_position: UncheckedAccount<'info>,

    /// CHECK: Account to mark the lower tick as initialized
    #[account(
        mut, 
    )]
    pub tick_array_lower: UncheckedAccount<'info>,

    /// CHECK:Account to store data for the position's upper tick
    #[account(
        mut, 
    )]
    pub tick_array_upper: UncheckedAccount<'info>,

    /// CHECK: personal position state
    #[account(
        mut, 
    )]
    pub personal_position: UncheckedAccount<'info>,

    // / The token_0 account deposit token to the pool
    #[account(
        mut,
        token::mint = token_vault_0.mint
    )]
    pub token_account_0: Box<InterfaceAccount<'info, TokenAccount>>,

    // / The token_1 account deposit token to the pool
    #[account(
        mut,
        token::mint = token_vault_1.mint
    )]
    pub token_account_1: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The address that holds pool tokens for token_0
    #[account(
        mut,
        constraint = token_vault_0.key() == pool_state.load()?.token_vault_0
    )]
    pub token_vault_0: Box<InterfaceAccount<'info, TokenAccount>>,

    // / The address that holds pool tokens for token_1
    #[account(
        mut,
        constraint = token_vault_1.key() == pool_state.load()?.token_vault_1
    )]
    pub token_vault_1: Box<InterfaceAccount<'info, TokenAccount>>,

    /// The mint of token vault 0
    #[account(address = token_vault_0.mint)]
    pub vault_0_mint: Box<InterfaceAccount<'info, Mint>>,
    /// The mint of token vault 1
    #[account(address = token_vault_1.mint)]
    pub vault_1_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Sysvar for token mint and ATA creation
    pub rent: Sysvar<'info, Rent>,

    /// Program to create the position manager state account
    pub system_program: Program<'info, System>,

    /// Program to create mint account and mint tokens
    pub token_program: Program<'info, Token>,
    /// Program to create an ATA for receiving position NFT
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// Program to create NFT metadata
    /// CHECK: Metadata program address constraint applied
    pub metadata_program: Program<'info, Metadata>,
    /// Program to create mint account and mint tokens
    pub token_program_2022: Program<'info, Token2022>,
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

pub fn updated_liquidity_personal_position<'info>(acc: AccountInfo<'info>) -> Result<u128> {
    let mut data: &[u8] = &acc.try_borrow_data()?;
    let state = PersonalPositionState::try_deserialize(&mut data)?;
    Ok(state.liquidity)
}

pub fn updated_liquidity_position<'info>(acc: AccountInfo<'info>) -> Result<u128> {
    let mut data: &[u8] = &acc.try_borrow_data()?;
    let state = ProtocolPositionState::try_deserialize(&mut data)?;
    Ok(state.liquidity)
}

impl<'info> OpenPositionCtx<'info> {
    pub fn open_position<'a, 'b, 'c: 'info>(
        &mut self,
        amount_0_max: u64,
        amount_1_max: u64,
        tick_lower_index: i32,
        tick_upper_index: i32,
        tick_array_lower_start_index: i32,
        tick_array_upper_start_index: i32,
        remaining_accounts: &'c [AccountInfo<'info>]
    ) -> Result<()> {
        self.dex_open_position(
            tick_lower_index,
            tick_upper_index,
            tick_array_lower_start_index,
            tick_array_upper_start_index,
            0,
            amount_0_max,
            amount_1_max,
            remaining_accounts
        )?;
        let liquidity = updated_liquidity_personal_position(
            self.personal_position.to_account_info()
        )?;

        let pool_position = &mut self.pool_position;
        let manager_account = &mut self.manager_account;

        pool_position.pool_position_nft_key = self.position_nft_account.key();
        pool_position.position_nft_mint_key = self.position_nft_mint.key();
        pool_position.position_nft_account_key = self.position_nft_account.key();
        pool_position.liquidity = liquidity;

        manager_account.init_liquidity = liquidity;
        manager_account.liquidity = liquidity;

        msg!("Pool Position NFT : {:?}", self.position_nft_mint.key());

        Ok(())
    }

    fn dex_open_position<'a, 'b, 'c: 'info>(
        &mut self,
        tick_lower_index: i32,
        tick_upper_index: i32,
        tick_array_lower_start_index: i32,
        tick_array_upper_start_index: i32,
        liquidity: u128,
        amount_0_max: u64,
        amount_1_max: u64,
        remaining_accounts: &'c [AccountInfo<'info>]
    ) -> Result<()> {
        let cpi_accounts = cpi::accounts::OpenPositionV2 {
            payer: self.manager.to_account_info(),
            position_nft_owner: self.pool_position.to_account_info(),
            position_nft_mint: self.position_nft_mint.to_account_info(),
            position_nft_account: self.position_nft_account.to_account_info(),
            metadata_account: self.metadata_account.to_account_info(),
            pool_state: self.pool_state.to_account_info(),
            protocol_position: self.protocol_position.to_account_info(),
            tick_array_lower: self.tick_array_lower.to_account_info(),
            tick_array_upper: self.tick_array_upper.to_account_info(),
            personal_position: self.personal_position.to_account_info(),
            token_account_0: self.token_account_0.to_account_info(),
            token_account_1: self.token_account_1.to_account_info(),
            token_vault_0: self.token_vault_0.to_account_info(),
            token_vault_1: self.token_vault_1.to_account_info(),
            rent: self.rent.to_account_info(),
            system_program: self.system_program.to_account_info(),
            token_program: self.token_program.to_account_info(),
            associated_token_program: self.associated_token_program.to_account_info(),
            metadata_program: self.metadata_program.to_account_info(),
            token_program_2022: self.token_program_2022.to_account_info(),
            vault_0_mint: self.vault_0_mint.to_account_info(),
            vault_1_mint: self.vault_1_mint.to_account_info(),
        };

        let cpi_context = CpiContext::new(
            self.clmm_program.to_account_info(),
            cpi_accounts
        ).with_remaining_accounts(remaining_accounts.to_vec());

        cpi::open_position_v2(
            cpi_context,
            tick_lower_index,
            tick_upper_index,
            tick_array_lower_start_index,
            tick_array_upper_start_index,
            liquidity,
            amount_0_max,
            amount_1_max,
            false,
            Some(true)
        )
    }
}
