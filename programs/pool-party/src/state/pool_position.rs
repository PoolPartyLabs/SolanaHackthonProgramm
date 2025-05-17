use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct PoolPositionConfig {
    pub bump: u8,

    pub pda_bump: u8,

    pub tick_lower_index: i32,

    pub tick_upper_index: i32,

    // / The third party pool key (e.g. Raydium pool state key)
    pub pool_key: Pubkey,

    pub pool_position_key: Pubkey,

    pub manager_key: Pubkey,

    pub token_vault_0_key: Pubkey,

    pub token_vault_1_key: Pubkey,

    pub vault_0_mint_key: Pubkey,

    pub vault_1_mint_key: Pubkey,

    #[max_len(32)]
    pub name: String,
}

impl PoolPositionConfig {
    pub const LEN: usize = 8 + PoolPositionConfig::INIT_SPACE;

    /// Seed to derive account address and signature
    pub const POOL_POSITION_CONFIG_SEED: &'static str = "pool_position_config:";
}

#[account]
#[derive(InitSpace)]
pub struct PoolPosition {
    pub bump: u8,

    pub pool_position_nft_account_bump: u8,

    pub pool_position_config_key: Pubkey,

    pub manager_key: Pubkey,

    pub pool_position_nft_key: Pubkey,

    pub position_nft_mint_key: Pubkey,

    pub position_nft_account_key: Pubkey,

    #[max_len(32)]
    pub name: String,

    pub fees_index0: u128,

    pub fees_index1: u128,

    pub liquidity: u128,

    pub vaults_initialized: bool,

    pub created_at: u64,
}

impl PoolPosition {
    pub const LEN: usize = 8 + PoolPosition::INIT_SPACE;

    /// Seed to derive account address and signature
    pub const POOL_POSITION_SEED: &'static str = "pool_position:";

    pub const POOL_POSITION_NFT_SEED: &'static str = "pool_position_nft:";

    pub const POOL_POSITION_VAULT_0_SEED: &'static str = "pool_position_vault_0:";

    pub const POOL_POSITION_VAULT_1_SEED: &'static str = "pool_position_vault_1:";

    pub const POOL_POSITION_FEES_VAULT_0_SEED: &'static str = "pool_pos_fees_vault_0:";

    pub const POOL_POSITION_FEES_VAULT_1_SEED: &'static str = "pool_pos_fees_vault_1:";
}
