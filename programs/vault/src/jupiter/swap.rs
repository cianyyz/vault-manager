use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use jupiter_interface::{
    instructions::{RouteIxArgs, RouteKeys},
    typedefs::{RoutePlanStep, Swap},
};

#[derive(Accounts)]
pub struct JupiterSwapAccounts<'info> {
    pub jupiter_program: UncheckedAccount<'info>,
    
    /// Token account to swap from
    pub source_token_account: Account<'info, TokenAccount>,
    
    /// Token account to swap to  
    pub destination_token_account: Account<'info, TokenAccount>,
    
    /// Authority that owns the source tokens
    pub authority: Signer<'info>,
    
    /// Token program
    pub token_program: Program<'info, anchor_spl::token::Token>,
}

pub fn jupiter_swap(
    ctx: Context<JupiterSwapAccounts>, 
    amount_in: u64,
    minimum_amount_out: u64,
    route_plan: Vec<RoutePlanStep>,
) -> Result<()> {
    // Create the route keys
    let route_keys = RouteKeys {
        token_program: ctx.accounts.token_program.key(),
        user_transfer_authority: ctx.accounts.authority.key(),
        user_source_token_account: ctx.accounts.source_token_account.key(),
        user_destination_token_account: ctx.accounts.destination_token_account.key(),
        program: ctx.accounts.jupiter_program.key(),
        // Default values for remaining required fields
        platform_fee_account: Pubkey::default(),
        destination_token_account: ctx.accounts.destination_token_account.key(),
        destination_mint: ctx.accounts.destination_token_account.mint,
        event_authority: Pubkey::default(),
    };

    // Create the route arguments
    let route_args = RouteIxArgs {
        route_plan,
        in_amount: amount_in,
        quoted_out_amount: minimum_amount_out,
        slippage_bps: 50, // 0.5% slippage
        platform_fee_bps: 0,
    };

    // Create and execute the route instruction
    let ix = jupiter_interface::instructions::route_ix(
        route_keys,
        route_args,
    )?;

    // Execute the swap via CPI
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.jupiter_program.to_account_info(),
            ctx.accounts.source_token_account.to_account_info(),
            ctx.accounts.destination_token_account.to_account_info(),
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
    )?;

    Ok(())
}

// Helper function to build a simple token swap route
pub fn build_token_swap_route(
    amount_in: u64,
    minimum_amount_out: u64,
    swap: Swap,
) -> Vec<RoutePlanStep> {
    vec![RoutePlanStep {
        percent: 100, // Use 100% of input amount
        input_index: 0,
        output_index: 0,
        swap,
    }]
} 