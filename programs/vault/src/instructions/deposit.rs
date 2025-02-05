use anchor_lang::prelude::*;
use anchor_spl::{
    token::{self, Mint, Token, TokenAccount},
    associated_token::AssociatedToken,
};

use crate::state::*;

pub fn deposit_usdc(
    ctx: Context<DepositUsdc>,
    amount: u64
) -> Result<()> {
    // Calculate shares to mint based on total shares and vault balance
    let shares_to_mint = if ctx.accounts.vault.total_shares == 0 {
        // Initial deposit - 1:1 ratio
        amount
    } else {
        // Calculate based on proportion of existing shares
        let vault_balance = ctx.accounts.vault_usdc.amount;
        amount
            .checked_mul(ctx.accounts.vault.total_shares)
            .and_then(|v| v.checked_div(vault_balance))
            .ok_or(VaultError::CalculationError)?
    };

    // Transfer USDC to vault
    let cpi_accounts = token::Transfer {
        from: ctx.accounts.depositor_usdc.to_account_info(),
        to: ctx.accounts.vault_usdc.to_account_info(),
        authority: ctx.accounts.depositor.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, amount)?;

    // Update vault state
    let vault = &mut ctx.accounts.vault;
    vault.total_shares = vault.total_shares
        .checked_add(shares_to_mint)
        .ok_or(VaultError::CalculationError)?;

    // Update signer seeds to match PDA derivation
    let seeds = &[
        b"vault".as_ref(),
        ctx.accounts.vault.owner.as_ref(),
        ctx.accounts.vault.share_mint.as_ref(),
        &[ctx.accounts.vault.bumps.vault],
    ];
    let signer = &[&seeds[..]];

    // Mint share tokens
    let cpi_accounts = token::MintTo {
        mint: ctx.accounts.share_mint.to_account_info(),
        to: ctx.accounts.depositor_share_account.to_account_info(),
        authority: ctx.accounts.vault.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
    token::mint_to(cpi_ctx, shares_to_mint)?;

    Ok(())
}

#[derive(Accounts)]
pub struct DepositUsdc<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            b"vault",
            vault.owner.as_ref(),
            vault.share_mint.as_ref()
        ],
        bump = vault.bumps.vault
    )]
    pub vault: Account<'info, Vault>,
    
    #[account(mut)]
    pub vault_usdc: Account<'info, TokenAccount>,
    
    #[account(mut)]
    pub depositor_usdc: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = share_mint.key() == vault.share_mint
    )]
    pub share_mint: Account<'info, Mint>,
    
    #[account(
        init_if_needed,
        payer = depositor,
        associated_token::mint = share_mint,
        associated_token::authority = depositor
    )]
    pub depositor_share_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}