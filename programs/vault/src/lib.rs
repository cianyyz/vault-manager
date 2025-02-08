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
mod jupiter;

use instructions::*;
use state::*;
use orca::*;
use jupiter::*;


declare_id!("EY1mGiYQgaLZW2fNNUjYZRNRqjDgXGZAST1endGDxjKP");

//Replace with your own wallet keypair
const OWNER: &str = "Ct76ND8eC3MZ6PPHNNvMmz7Q8K18sobGdz6t3gyC63Pf";

/// Main program module containing all instruction handlers
#[program]
pub mod vault {
    use super::*;

    /// Initializes a new vault
    /// 
    /// This instruction creates a new vault account and performs the initial USDC deposit.
    /// The first depositor becomes the vault administrator.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    /// * `deposit_amount` - Initial USDC deposit amount
    pub fn initialize(ctx: Context<Initialize>, deposit_amount: u64) -> Result<()> {
        instructions::initialize::initialize(ctx, deposit_amount)
    }

    /// Deposits USDC into the vault
    /// 
    /// Users can deposit USDC and receive vault shares in return. The share amount
    /// is calculated based on the current vault total value and existing shares.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    /// * `amount` - Amount of USDC to deposit
    pub fn deposit_usdc(ctx: Context<DepositUsdc>, amount: u64) -> Result<()> {
        instructions::deposit::deposit_usdc(ctx, amount)
    }

    /// Withdraws USDC from the vault
    /// 
    /// Users can burn their vault shares to withdraw proportional USDC amount.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    /// * `shares_amount` - Amount of vault shares to burn for withdrawal
    pub fn withdraw_usdc(ctx: Context<WithdrawUsdc>, shares_amount: u64) -> Result<()> {
        instructions::withdraw::withdraw_usdc(ctx, shares_amount)
    }

    /// Adds a token to the whitelist
    /// 
    /// Only the vault owner can add tokens to the whitelist. Whitelisted tokens
    /// can be used in vault operations.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    /// * `token_mint` - Public key of the token mint to whitelist
    pub fn add_whitelisted_token(ctx: Context<ManageTokenWhitelist>, token_mint: Pubkey) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        instructions::manage_token_whitelist::add_whitelisted_token(ctx, token_mint)
    }

    /// Removes a token from the whitelist
    /// 
    /// Only the vault owner can remove tokens from the whitelist.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    /// * `token_mint` - Public key of the token mint to remove from whitelist
    pub fn remove_whitelisted_token(ctx: Context<ManageTokenWhitelist>, token_mint: Pubkey) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        instructions::manage_token_whitelist::remove_whitelisted_token(ctx, token_mint)
    }

    /// Verifies the Orca Whirlpools config account
    /// 
    /// Admin-only instruction to verify and whitelist a Whirlpools config account.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing the config account to verify
    pub fn verify_whirlpools_config_account(ctx: Context<VerifyWhirlpoolsConfigAccount>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::verify_account::handler_whirlpools_config(ctx)
    }

    /// Verifies an Orca fee tier account
    /// 
    /// Admin-only instruction to verify and whitelist a fee tier account.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing the fee tier account to verify
    pub fn verify_feetier_account(ctx: Context<VerifyFeeTierAccount>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::verify_account::handler_feetier(ctx)
    }

    /// Verifies an Orca whirlpool account
    /// 
    /// Admin-only instruction to verify and whitelist a whirlpool account.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing the whirlpool account to verify
    pub fn verify_whirlpool_account(ctx: Context<VerifyWhirlpoolAccount>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::verify_account::handler_whirlpool(ctx)
    }

    /// Verifies Orca tick array accounts
    /// 
    /// Admin-only instruction to verify and whitelist tick array accounts.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing the tick array account to verify
    /// * `sampling1` through `sampling8` - Tick array sampling parameters
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

    /// Verifies an Orca position account
    /// 
    /// Admin-only instruction to verify and whitelist a position account.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing the position account to verify
    pub fn verify_position_account(ctx: Context<VerifyPositionAccount>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::verify_account::handler_position(ctx)
    }

    /// Executes a swap through Orca Whirlpool
    /// 
    /// Admin-only instruction to perform a token swap using Orca's Whirlpool DEX.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    /// * `amount` - Amount of tokens to swap
    /// * `other_amount_threshold` - Minimum amount of tokens to receive
    /// * `sqrt_price_limit` - Price limit for the swap
    /// * `amount_specified_is_input` - Whether the amount specified is input or output
    /// * `a_to_b` - Direction of the swap (token A to B or B to A)
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

    /// Opens a new position in an Orca Whirlpool
    /// 
    /// Admin-only instruction to open a new liquidity position.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    /// * `tick_lower_index` - Lower tick index of the position
    /// * `tick_upper_index` - Upper tick index of the position
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

    /// Increases liquidity in an existing Orca position
    /// 
    /// Admin-only instruction to add liquidity to a position.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    /// * `liquidity` - Amount of liquidity to add
    /// * `token_max_a` - Maximum amount of token A to add
    /// * `token_max_b` - Maximum amount of token B to add
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

    /// Decreases liquidity in an existing Orca position
    /// 
    /// Admin-only instruction to remove liquidity from a position.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    /// * `liquidity` - Amount of liquidity to remove
    /// * `token_min_a` - Minimum amount of token A to receive
    /// * `token_min_b` - Minimum amount of token B to receive
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

    /// Updates fees and rewards for an Orca position
    /// 
    /// Admin-only instruction to update and collect fees and rewards.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    pub fn proxy_update_fees_and_rewards(ctx: Context<ProxyUpdateFeesAndRewards>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::proxy_update_fees_and_rewards::handler(ctx)
    }

    /// Collects accumulated fees from an Orca position
    /// 
    /// Admin-only instruction to collect trading fees.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    pub fn proxy_collect_fees(ctx: Context<ProxyCollectFees>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::proxy_collect_fees::handler(ctx)
    }

    /// Collects rewards from an Orca position
    /// 
    /// Admin-only instruction to collect liquidity mining rewards.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    /// * `reward_index` - Index of the reward to collect
    pub fn proxy_collect_reward(ctx: Context<ProxyCollectReward>, reward_index: u8) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::proxy_collect_reward::handler(ctx, reward_index)
    }

    /// Closes an Orca position
    /// 
    /// Admin-only instruction to close a liquidity position.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    pub fn proxy_close_position(ctx: Context<ProxyClosePosition>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.signer_account.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        orca::proxy_close_position::handler(ctx)
    }

    /// Initializes a new Orca Whirlpool
    /// 
    /// Admin-only instruction to create a new liquidity pool.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    /// * `tick_spacing` - Tick spacing for the new pool
    /// * `initial_sqrt_price` - Initial square root price for the pool
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

    /// Initializes a new tick array for an Orca Whirlpool
    /// 
    /// Admin-only instruction to initialize a tick array.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    /// * `start_tick_index` - Starting tick index for the array
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

    /// Executes a token swap through Jupiter Aggregator
    /// 
    /// Admin-only instruction to perform a token swap using Jupiter's aggregated routes.
    ///
    /// # Parameters
    /// * `ctx` - Context object containing all required accounts
    /// * `amount_in` - Amount of input tokens to swap
    /// * `minimum_amount_out` - Minimum amount of output tokens to receive
    /// * `route_plan` - Jupiter swap route plan to execute
    pub fn proxy_jupiter_swap(
        ctx: Context<JupiterSwapAccounts>,
        amount_in: u64,
        minimum_amount_out: u64,
        route_plan: Vec<jupiter_interface::typedefs::RoutePlanStep>,
    ) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.authority.key(),
            OWNER.parse::<Pubkey>().unwrap(),
            VaultError::UnauthorizedAccess
        );
        swap::jupiter_swap(ctx, amount_in, minimum_amount_out, route_plan)
    }
}

