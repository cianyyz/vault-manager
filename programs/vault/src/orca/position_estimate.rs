use anchor_lang::prelude::*;
use whirlpool_cpi::{self, state::{Whirlpool, Position}};
use crate::state::*;

pub struct PositionValue<'info> {
    pub position: &'info Position,
    pub whirlpool: &'info Whirlpool,
}

// Helper functions for price math
fn get_sqrt_price_at_tick(tick: i32) -> Result<u128> {
    let abs_tick = tick.abs() as i32;
    if abs_tick > 443636 { // Max tick allowed
        return Err(VaultError::CalculationError.into());
    }
    
    let mut price = if abs_tick & 0x1 != 0 {
        "fffcb933bd6fad37aa2d162d1a594001".parse::<u128>().unwrap()
    } else {
        "100000000000000000000000000000000".parse::<u128>().unwrap()
    };

    if abs_tick & 0x2 != 0 {
        price = price
            .checked_mul("fff97272373d413259a46990580e213a".parse::<u128>().unwrap())
            .ok_or(VaultError::CalculationError)?
            .checked_shr(128)
            .ok_or(VaultError::CalculationError)?;
    }
    // ... more bit checks for precision
    // Full implementation would check all bits

    if tick > 0 {
        price = u128::MAX / price;
    }

    Ok(price)
}

fn get_amounts_from_liquidity(
    liquidity: u128,
    sqrt_price_current: u128,
    tick_lower: i32,
    tick_upper: i32,
    round_up: bool,
) -> Result<(u64, u64)> {
    let sqrt_price_lower = get_sqrt_price_at_tick(tick_lower)?;
    let sqrt_price_upper = get_sqrt_price_at_tick(tick_upper)?;
    
    if sqrt_price_current <= sqrt_price_lower {
        // Position is entirely below current price
        // Only token0 (amount_a)
        let amount_a = get_amount0_delta(
            liquidity,
            sqrt_price_lower,
            sqrt_price_upper,
            round_up
        )?;
        Ok((amount_a, 0))
    } else if sqrt_price_current < sqrt_price_upper {
        // Position straddles current price
        // Both tokens
        let amount_a = get_amount0_delta(
            liquidity,
            sqrt_price_current,
            sqrt_price_upper,
            round_up
        )?;
        let amount_b = get_amount1_delta(
            liquidity,
            sqrt_price_lower,
            sqrt_price_current,
            round_up
        )?;
        Ok((amount_a, amount_b))
    } else {
        // Position is entirely above current price
        // Only token1 (amount_b)
        let amount_b = get_amount1_delta(
            liquidity,
            sqrt_price_lower,
            sqrt_price_upper,
            round_up
        )?;
        Ok((0, amount_b))
    }
}

fn get_amount0_delta(
    liquidity: u128,
    sqrt_price_lower: u128,
    sqrt_price_upper: u128,
    round_up: bool,
) -> Result<u64> {
    if sqrt_price_lower > sqrt_price_upper {
        return Err(VaultError::CalculationError.into());
    }

    let numerator = liquidity << 64;
    let denominator = sqrt_price_upper;

    let amount = if round_up {
        (numerator / denominator + 1) as u64
    } else {
        (numerator / denominator) as u64
    };

    Ok(amount)
}

fn get_amount1_delta(
    liquidity: u128,
    sqrt_price_lower: u128,
    sqrt_price_upper: u128,
    round_up: bool,
) -> Result<u64> {
    if sqrt_price_lower > sqrt_price_upper {
        return Err(VaultError::CalculationError.into());
    }

    let amount = if round_up {
        ((liquidity * (sqrt_price_upper - sqrt_price_lower)) >> 64) + 1
    } else {
        (liquidity * (sqrt_price_upper - sqrt_price_lower)) >> 64
    };

    Ok(amount as u64)
}

//We will assume all positions are TOKEN/USD
pub fn get_position_value(position_value: &PositionValue) -> Result<u64> {
    let position = position_value.position;
    let whirlpool = position_value.whirlpool;
    
    let sqrt_price_x64 = whirlpool.sqrt_price;
    let price = sqrt_price_x64
        .checked_mul(sqrt_price_x64)
        .ok_or(VaultError::CalculationError)?
        .checked_shr(128)
        .ok_or(VaultError::CalculationError)?;
    
    let (amount_a, amount_b) = get_amounts_from_liquidity(
        position.liquidity,
        sqrt_price_x64,
        position.tick_lower_index,
        position.tick_upper_index,
        true
    )?;

    let value_from_a = (amount_a as u128)
        .checked_mul(price)
        .ok_or(VaultError::CalculationError)?;
    
    let total_value = value_from_a
        .checked_add(amount_b as u128)
        .ok_or(VaultError::CalculationError)?;

    Ok(total_value.try_into().map_err(|_| VaultError::CalculationError)?)
}