use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};
use crate::state::*;
use crate::state::VaultBumps;

pub fn initialize(
    ctx: Context<Initialize>, deposit_amount: u64
) -> Result<()> {

    // ensure deposit amount is greater than 0
    if deposit_amount <= 0 {
        return err!(VaultError::InvalidDepositAmount);
    }

    // Transfer token from the vault owner to the vault token account
    let context = ctx.accounts.token_program_context(Transfer {
        from: ctx.accounts.owner_token_account.to_account_info(),
        to: ctx.accounts.vault_token_account.to_account_info(),
        authority: ctx.accounts.owner.to_account_info(),
    });
    transfer(context, deposit_amount)?;

    let bumps = VaultBumps {
        vault: ctx.bumps.vault,
        vault_authority: ctx.bumps.vault_authority,
        vault_token_account: ctx.bumps.vault_token_account,
    };

    ctx.accounts.vault.set_inner(Vault {
        total_shares: 0,
        current_position: None,
        current_whirlpool: None,
        owner: ctx.accounts.owner.key(),
        share_mint: ctx.accounts.mint.key(),
        whitelisted_tokens: Vec::new(),
        bumps
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    owner: Signer<'info>,
    
    #[account(constraint = mint.is_initialized == true)]
    mint: Account<'info, Mint>,

    #[account(mut, token::mint=mint, token::authority=owner)]
    pub owner_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = owner,
        space = Vault::LEN,
        seeds = [b"vault".as_ref(), owner.key().as_ref(), mint.key().as_ref()],
        bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        seeds = [b"authority".as_ref(), vault.key().as_ref()], bump
    )]
    vault_authority: SystemAccount<'info>,
    
    #[account(
        init,
        payer = owner,
        token::mint = mint,
        token::authority = vault_authority,
        seeds = [b"tokens", vault.key().as_ref()],
        bump
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>
}

impl<'info> Initialize<'info> {
    pub fn token_program_context<T: ToAccountMetas + ToAccountInfos<'info>>(
        &self,
        data: T,
    ) -> CpiContext<'_, '_, '_, 'info, T> {
        CpiContext::new(self.token_program.to_account_info(), data)
    }
}