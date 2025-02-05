use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

use crate::state::*;

// Fee constants (in basis points - 1/100th of a percent)
pub const OWNER_FEE_BPS: u64 = 50; // 0.5%
pub const BURN_FEE_BPS: u64 = 100; // 1%

pub fn withdraw_usdc(
    ctx: Context<WithdrawUsdc>,
    shares_amount: u64
) -> Result<()> {
    // Calculate fees
    let owner_fee = shares_amount.checked_mul(OWNER_FEE_BPS)
        .ok_or(VaultError::CalculationError)?
        .checked_div(10000)
        .ok_or(VaultError::CalculationError)?;
    
    let burn_fee = shares_amount.checked_mul(BURN_FEE_BPS)
        .ok_or(VaultError::CalculationError)?
        .checked_div(10000)
        .ok_or(VaultError::CalculationError)?;
    
    let shares_to_redeem = shares_amount
        .checked_sub(owner_fee)
        .ok_or(VaultError::CalculationError)?
        .checked_sub(burn_fee)
        .ok_or(VaultError::CalculationError)?;

    // Calculate USDC amount to return based on share proportion
    let vault_balance = ctx.accounts.vault_usdc.amount;
    let usdc_amount = shares_to_redeem
        .checked_mul(vault_balance)
        .and_then(|v| v.checked_div(ctx.accounts.vault.total_shares))
        .ok_or(VaultError::CalculationError)?;

    // Transfer USDC from vault to withdrawer
    let seeds = &[
        b"vault".as_ref(),
        &[ctx.accounts.vault.bumps.vault],
    ];
    let signer = &[&seeds[..]];
    
    let cpi_accounts = token::Transfer {
        from: ctx.accounts.vault_usdc.to_account_info(),
        to: ctx.accounts.withdrawer_usdc.to_account_info(),
        authority: ctx.accounts.vault.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::transfer(cpi_ctx, usdc_amount)?;

    // Transfer owner fee shares to owner
    let cpi_accounts = token::Transfer {
        from: ctx.accounts.withdrawer_share_account.to_account_info(),
        to: ctx.accounts.owner_share_account.to_account_info(),
        authority: ctx.accounts.withdrawer.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
    );
    token::transfer(cpi_ctx, owner_fee)?;

    // Burn remaining fee shares
    let cpi_accounts = token::Burn {
        mint: ctx.accounts.share_mint.to_account_info(),
        from: ctx.accounts.withdrawer_share_account.to_account_info(),
        authority: ctx.accounts.withdrawer.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        cpi_accounts,
    );
    token::burn(cpi_ctx, burn_fee)?;

    // Update vault state
    let vault = &mut ctx.accounts.vault;
    vault.total_shares = vault.total_shares
        .checked_sub(burn_fee)  // Only subtract burned shares, owner fee shares still exist
        .ok_or(VaultError::CalculationError)?;

    Ok(())
}

#[derive(Accounts)]
pub struct WithdrawUsdc<'info> {
    #[account(mut)]
    pub withdrawer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"vault"],
        bump = vault.bumps.vault,
    )]
    pub vault: Account<'info, Vault>,
    
    /// The vault owner who receives fee shares
    pub vault_owner: SystemAccount<'info>,
    
    #[account(
        mut,
        associated_token::mint = share_mint,
        associated_token::authority = vault_owner
    )]
    pub owner_share_account: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub vault_usdc: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub withdrawer_usdc: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub share_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        associated_token::mint = share_mint,
        associated_token::authority = withdrawer
    )]
    pub withdrawer_share_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
}
