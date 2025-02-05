#[error_code]
pub enum ErrorCode {
    #[msg("Failed to calculate position value")]
    CalculationError,
    // ... other error variants
}

