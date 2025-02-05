use anchor_lang::prelude::*;


pub const MAX_WHITELISTED_TOKENS: usize = 10;

#[account]
#[derive(Debug)]
pub struct Vault {
    pub owner: Pubkey,
    pub share_mint: Pubkey,
    pub bumps: VaultBumps,
    pub total_shares: u64,
    pub current_position: Option<Pubkey>,
    pub current_whirlpool: Option<Pubkey>,
    pub whitelisted_tokens: Vec<Pubkey>,
}

impl Vault {
    pub const LEN: usize = {
        let discriminator = 8;
        let amounts = MAX_WHITELISTED_TOKENS * 8;
        let options = 2 * (1 + 32);
        let initialized = 1;
        let pubkeys = 2 * 32;
        let vault_bumps = 3 * 1;
        discriminator + amounts + options + initialized + pubkeys + vault_bumps
    };
}


#[derive(AnchorDeserialize, AnchorSerialize, Debug, Clone)]
pub struct VaultBumps {
    pub vault: u8,
    pub vault_authority: u8,
    pub vault_token_account: u8,
}

#[error_code]
pub enum VaultError {
    #[msg("Invalid fee percentage")]
    InvalidFeePercentage,
    #[msg("Token is not whitelisted")]
    TokenNotWhitelisted,
    #[msg("Unauthorized token account")]
    UnauthorizedToken,
    #[msg("Invalid token white list")]
    InvalidTokenWhitelist,
    #[msg("Position or whirlpool mismatch with vault state")]
    PositionMismatch,
    #[msg("Calculation failed due to overflow or division by zero")]
    CalculationError,
    #[msg("Maximum number of whitelisted tokens reached")]
    MaxWhitelistedTokensReached,
    #[msg("Insufficient liquidity")]
    InsufficientLiquidity,
    #[msg("Invalid whirlpool - must be token/USDC pair")]
    InvalidWhirlpool,
    #[msg("Missing whirlpool for token price")]
    MissingWhirlpool,
    #[msg("Unauthorized access - only owner can perform this action")]
    UnauthorizedAccess,
    #[msg("Deposit amount must be greater than 0")]
    InvalidDepositAmount,
    #[msg("Withdraw amount must be an amount available in the vault")]
    InvalidWithdrawAmount,
}

pub mod estimate;
pub use estimate::*; 