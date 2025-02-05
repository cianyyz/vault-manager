#![allow(unexpected_cfgs)]

//! Vault Program
//! 
//! A Solana program that implements a vault system for managing deposits, withdrawals,
//! and interactions with Orca Whirlpool DEX. The vault allows users to deposit USDC,
//! receive shares, and participate in automated DeFi strategies.

use anchor_lang::prelude::*;


mod instructions;
mod state;
mod orca;

use instructions::*;
use state::*;
use orca::*;


declare_id!("EY1mGiYQgaLZW2fNNUjYZRNRqjDgXGZAST1endGDxjKP");

//Replace with your own wallet keypair
const OWNER: &str = "Ct76ND8eC3MZ6PPHNNvMmz7Q8K18sobGdz6t3gyC63Pf";

/// Main program module containing all instruction handlers
#[program]
pub mod vault {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, deposit_amount: u64) -> Result<()> {
        instructions::initialize::initialize(ctx, deposit_amount)
    }

    pub fn deposit_usdc(ctx: Context<DepositUsdc>, amount: u64) -> Result<()> {
        instructions::deposit::deposit_usdc(ctx, amount)
    }

    pub fn withdraw_usdc(ctx: Context<WithdrawUsdc>, shares_amount: u64) -> Result<()> {
        instructions::withdraw::withdraw_usdc(ctx, shares_amount)
    }

    pub fn add_whitelisted_token(ctx: Context<ManageTokenWhitelist>, token_mint: Pubkey) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        instructions::manage_token_whitelist::add_whitelisted_token(ctx, token_mint)
    }

    pub fn remove_whitelisted_token(ctx: Context<ManageTokenWhitelist>, token_mint: Pubkey) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        instructions::manage_token_whitelist::remove_whitelisted_token(ctx, token_mint)
    }

    pub fn verify_whirlpools_config_account(ctx: Context<VerifyWhirlpoolsConfigAccount>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::verify_account::handler_whirlpools_config(ctx)
    }

    pub fn verify_feetier_account(ctx: Context<VerifyFeeTierAccount>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::verify_account::handler_feetier(ctx)
    }

    pub fn verify_whirlpool_account(ctx: Context<VerifyWhirlpoolAccount>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::verify_account::handler_whirlpool(ctx)
    }

    pub fn verify_tickarray_account(
        ctx: Context<VerifyTickArrayAccount>,
        sampling1: u32,
        sampling2: u32,
        sampling3: u32,
        sampling4: u32,
        sampling5: u32,
        sampling6: u32,
        sampling7: u32,
        sampling8: u32,
    ) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::verify_account::handler_tickarray(
            ctx,
            sampling1, sampling2, sampling3, sampling4,
            sampling5, sampling6, sampling7, sampling8,
        )
    }

    pub fn verify_position_account(ctx: Context<VerifyPositionAccount>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::verify_account::handler_position(ctx)
    }

    pub fn proxy_swap(
        ctx: Context<ProxySwap>,
        amount: u64,
        other_amount_threshold: u64,
        sqrt_price_limit: u128,
        amount_specified_is_input: bool,
        a_to_b: bool,
    ) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::proxy_swap::handler(
            ctx,
            amount,
            other_amount_threshold,
            sqrt_price_limit,
            amount_specified_is_input,
            a_to_b,
        )
    }

    pub fn proxy_open_position(
        ctx: Context<ProxyOpenPosition>,
        tick_lower_index: i32,
        tick_upper_index: i32,
    ) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::proxy_open_position::handler(
            ctx,
            tick_lower_index,
            tick_upper_index,
        )
    }

    pub fn proxy_increase_liquidity(
        ctx: Context<ProxyIncreaseLiquidity>,
        liquidity: u128,
        token_max_a: u64,
        token_max_b: u64,
    ) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::proxy_increase_liquidity::handler(
            ctx,
            liquidity,
            token_max_a,
            token_max_b,
        )
    }

    pub fn proxy_decrease_liquidity(
        ctx: Context<ProxyDecreaseLiquidity>,
        liquidity: u128,
        token_min_a: u64,
        token_min_b: u64,
    ) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::proxy_decrease_liquidity::handler(
            ctx,
            liquidity,
            token_min_a,
            token_min_b,
        )
    }

    pub fn proxy_update_fees_and_rewards(ctx: Context<ProxyUpdateFeesAndRewards>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::proxy_update_fees_and_rewards::handler(ctx)
    }

    pub fn proxy_collect_fees(ctx: Context<ProxyCollectFees>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::proxy_collect_fees::handler(ctx)
    }

    pub fn proxy_collect_reward(ctx: Context<ProxyCollectReward>, reward_index: u8) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::proxy_collect_reward::handler(ctx, reward_index)
    }

    pub fn proxy_close_position(ctx: Context<ProxyClosePosition>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::proxy_close_position::handler(ctx)
    }

    pub fn proxy_initialize_pool(
        ctx: Context<ProxyInitializePool>,
        tick_spacing: u16,
        initial_sqrt_price: u128,
    ) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::proxy_initialize_pool::handler(
            ctx,
            tick_spacing,
            initial_sqrt_price,
        )
    }

    pub fn proxy_initialize_tick_array(
        ctx: Context<ProxyInitializeTickArray>,
        start_tick_index: i32,
    ) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::proxy_initialize_tick_array::handler(
            ctx,
            start_tick_index,
        )
    }
}

