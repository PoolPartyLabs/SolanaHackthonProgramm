use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid tick range")]
    InvalidTickRange,

    #[msg("sqrt_price_x64 out of range")]
    SqrtPriceX64,

    #[msg("The tick must be lesser than, or equal to the maximum tick(443636)")]
    TickUpperOverflow,
 
    InvalidTickArray,

    #[msg("The pool position vaults have already been initialized")]
    VaultsAlreadyInitialized,

    #[msg("Unauthorized")]
    Unauthorized,
}
