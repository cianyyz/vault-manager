use anchor_lang::prelude::*;
use crate::state::*;


pub fn add_whitelisted_token(
    ctx: Context<ManageTokenWhitelist>,
    token_mint: Pubkey,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    
    // Check if token is already whitelisted
    if vault.whitelisted_tokens.contains(&token_mint) {
        return Ok(());
    }

    // Check if we've reached the maximum number of whitelisted tokens
    if vault.whitelisted_tokens.len() >= MAX_WHITELISTED_TOKENS {
        return err!(VaultError::MaxWhitelistedTokensReached);
    }

    // Add the token to the whitelist
    vault.whitelisted_tokens.push(token_mint);
    Ok(())
}

pub fn remove_whitelisted_token(
    ctx: Context<ManageTokenWhitelist>,
    token_mint: Pubkey,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    
    // Remove the token from the whitelist if it exists
    if let Some(index) = vault.whitelisted_tokens.iter().position(|x| x == &token_mint) {
        vault.whitelisted_tokens.remove(index);
    }
    
    Ok(())
}

#[derive(Accounts)]
pub struct ManageTokenWhitelist<'info> {
    #[account(mut)]
    pub signer_account: Signer<'info>,

    #[account(
        mut,
        seeds = [b"vault"],
        bump = vault.bumps.vault,
    )]
    pub vault: Account<'info, Vault>,
}

