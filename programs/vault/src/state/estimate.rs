use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use whirlpool_cpi::state::{Whirlpool, Position};
use crate::state::*;
use crate::orca::position_estimate::PositionValue;

pub struct VaultEstimate<'info> {
    pub vault: Account<'info, Vault>,
    pub position: Box<Account<'info, Position>>,
    pub whirlpool: Box<Account<'info, Whirlpool>>,
    pub token_accounts: Vec<Account<'info, TokenAccount>>,
}

impl<'info> VaultEstimate<'info> {
    /// Get total vault value in USD (position value + token balances)
    pub fn get_vault_estimate(&self) -> Result<u64> {
        let mut total_value: u64 = 0;

        // If there's an active position, get its value
        if self.vault.current_position != None {
            let position_value = PositionValue {
                position: &self.position,
                whirlpool: &self.whirlpool,
            };
            
            let position_value = crate::orca::position_estimate::get_position_value(&position_value)?;
            total_value = total_value.checked_add(position_value).ok_or(VaultError::CalculationError)?;
        }

        // Add values from whitelisted token accounts
        for token_account in &self.token_accounts {
            if self.vault.whitelisted_tokens.contains(&token_account.mint) {
                // Assuming token amounts are in their native decimals (e.g., USDC with 6)
                total_value = total_value
                    .checked_add(token_account.amount)
                    .ok_or(VaultError::CalculationError)?;
            }
        }

        Ok(total_value)
    }

    /// Get per-share value in USD
    pub fn get_share_estimate(&self) -> Result<u64> {
        let total_value = self.get_vault_estimate()?;
        
        // If no shares exist, return 0 to prevent division by zero
        if self.vault.total_shares == 0 {
            return Ok(0);
        }

        // Calculate value per share (maintaining 6 decimal precision)
        total_value
            .checked_mul(1_000_000)  // Multiply by 10^6 to maintain precision
            .and_then(|v| v.checked_div(self.vault.total_shares))
            .ok_or(VaultError::CalculationError.into())
    }
} 