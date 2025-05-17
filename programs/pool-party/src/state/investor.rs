use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Investor {
    pub bump: u8,
    pub authority: Pubkey,
    pub pool_position_key: Pubkey,
    pub init_liquidity: u128,
    pub liquidity: u128,
    pub fees_earned0: u64,
    pub fees_earned1: u64,
    pub fees_index0: u128,
    pub fees_index1: u128,
    pub is_manager: bool,
}

impl Investor {
    pub const LEN: usize = 8 + std::mem::size_of::<Investor>();

    /// Seed to derive account address and signature
    pub const INVESTOR_SEED: &'static str = "investor:";

    pub const INVESTOR_DEPOSIT_STABLE_TOKEN_ACCOUNT_SEED: &'static str = "inv_stable_token_acct:";

    pub const INVESTOR_DEPOSIT_TOKEN_0_ACCOUNT_SEED: &'static str = "inv_dep_token_0_acct:";

    pub const INVESTOR_DEPOSIT_TOKEN_1_ACCOUNT_SEED: &'static str = "inv_dep_token_1_acct:";

    pub const INVESTOR_DEPOSIT_FEES_0_ACCOUNT_SEED: &'static str = "inv_dep_fees_0_acct:";

    pub const INVESTOR_DEPOSIT_FEES_1_ACCOUNT_SEED: &'static str = "inv_dep_fees_1_acct:";
}
