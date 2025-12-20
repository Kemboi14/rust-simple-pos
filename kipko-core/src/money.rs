//! Money module for precise financial calculations
//! 
//! This module provides a Money type that uses rust_decimal for precise
//! decimal arithmetic, avoiding floating-point inaccuracies in financial calculations.

use rust_decimal::{Decimal, prelude::FromPrimitive};
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Error type for money operations
#[derive(Debug, Error, PartialEq)]
pub enum MoneyError {
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),
    #[error("Currency mismatch: expected {expected}, got {actual}")]
    CurrencyMismatch { expected: String, actual: String },
    #[error("Arithmetic error: {0}")]
    ArithmeticError(String),
    #[error("Insufficient funds")]
    InsufficientFunds,
}

/// Result type for money operations
pub type MoneyResult<T> = Result<T, MoneyError>;

/// Money type for precise financial calculations
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Money {
    amount: Decimal,
    currency: String,
}

impl Money {
    /// Create a new Money instance
    pub fn new(amount: Decimal, currency: String) -> MoneyResult<Self> {
        if amount < dec!(0) {
            return Err(MoneyError::InvalidAmount("Amount cannot be negative".to_string()));
        }
        
        Ok(Self { amount, currency })
    }

    /// Create Money from a string amount
    pub fn from_str(amount: &str, currency: String) -> MoneyResult<Self> {
        let amount = amount.parse::<Decimal>()
            .map_err(|_| MoneyError::InvalidAmount(format!("Invalid amount: {}", amount)))?;
        Self::new(amount, currency)
    }

    /// Create Money from a float (for convenience, but discouraged)
    pub fn from_f64(amount: f64, currency: String) -> MoneyResult<Self> {
        let amount = Decimal::from_f64(amount)
            .ok_or_else(|| MoneyError::InvalidAmount("Invalid float amount".to_string()))?;
        Self::new(amount, currency)
    }

    /// Get the amount as Decimal
    pub fn amount(&self) -> Decimal {
        self.amount
    }

    /// Get the currency code
    pub fn currency(&self) -> &str {
        &self.currency
    }

    /// Add two Money instances (must have same currency)
    pub fn add(&self, other: &Money) -> MoneyResult<Money> {
        self.check_currency(other)?;
        Ok(Money {
            amount: self.amount + other.amount,
            currency: self.currency.clone(),
        })
    }

    /// Subtract two Money instances (must have same currency)
    pub fn subtract(&self, other: &Money) -> MoneyResult<Money> {
        self.check_currency(other)?;
        
        if self.amount < other.amount {
            return Err(MoneyError::InsufficientFunds);
        }
        
        Ok(Money {
            amount: self.amount - other.amount,
            currency: self.currency.clone(),
        })
    }

    /// Multiply Money by a factor
    pub fn multiply(&self, factor: Decimal) -> MoneyResult<Money> {
        if factor < dec!(0) {
            return Err(MoneyError::InvalidAmount("Factor cannot be negative".to_string()));
        }
        
        Ok(Money {
            amount: self.amount * factor,
            currency: self.currency.clone(),
        })
    }

    /// Calculate percentage of Money
    pub fn percentage(&self, percent: Decimal) -> MoneyResult<Money> {
        if percent < dec!(0) || percent > dec!(100) {
            return Err(MoneyError::InvalidAmount("Percent must be between 0 and 100".to_string()));
        }
        
        self.multiply(percent / dec!(100))
    }

    /// Check if two Money instances have the same currency
    fn check_currency(&self, other: &Money) -> MoneyResult<()> {
        if self.currency != other.currency {
            return Err(MoneyError::CurrencyMismatch {
                expected: self.currency.to_string(),
                actual: other.currency.to_string(),
            });
        }
        Ok(())
    }

    /// Check if Money is zero
    pub fn is_zero(&self) -> bool {
        self.amount == dec!(0)
    }

    /// Create zero Money
    pub fn zero(currency: String) -> Money {
        Money {
            amount: dec!(0),
            currency,
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.currency, self.amount)
    }
}

/// Common currency constants
pub mod currencies {
    pub const USD: &str = "USD";
    pub const EUR: &str = "EUR";
    pub const GBP: &str = "GBP";
    pub const JPY: &str = "JPY";
    pub const CAD: &str = "CAD";
    pub const AUD: &str = "AUD";
    
    pub fn usd() -> String { USD.to_string() }
    pub fn eur() -> String { EUR.to_string() }
    pub fn gbp() -> String { GBP.to_string() }
    pub fn jpy() -> String { JPY.to_string() }
    pub fn cad() -> String { CAD.to_string() }
    pub fn aud() -> String { AUD.to_string() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_money_creation() {
        let money = Money::new(dec!(100.50), currencies::usd()).unwrap();
        assert_eq!(money.amount(), dec!(100.50));
        assert_eq!(money.currency(), currencies::USD);
    }

    #[test]
    fn test_money_addition() {
        let money1 = Money::new(dec!(100.00), currencies::usd()).unwrap();
        let money2 = Money::new(dec!(50.25), currencies::usd()).unwrap();
        let result = money1.add(&money2).unwrap();
        assert_eq!(result.amount(), dec!(150.25));
    }

    #[test]
    fn test_money_subtraction() {
        let money1 = Money::new(dec!(100.00), currencies::usd()).unwrap();
        let money2 = Money::new(dec!(25.50), currencies::usd()).unwrap();
        let result = money1.subtract(&money2).unwrap();
        assert_eq!(result.amount(), dec!(74.50));
    }

    #[test]
    fn test_money_multiplication() {
        let money = Money::new(dec!(100.00), currencies::usd()).unwrap();
        let result = money.multiply(dec!(1.5)).unwrap();
        assert_eq!(result.amount(), dec!(150.00));
    }

    #[test]
    fn test_money_percentage() {
        let money = Money::new(dec!(200.00), currencies::usd()).unwrap();
        let result = money.percentage(dec!(25)).unwrap();
        assert_eq!(result.amount(), dec!(50.00));
    }

    #[test]
    fn test_currency_mismatch() {
        let usd = Money::new(dec!(100.00), currencies::usd()).unwrap();
        let eur = Money::new(dec!(100.00), currencies::eur()).unwrap();
        let result = usd.add(&eur);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MoneyError::CurrencyMismatch { .. }));
    }

    #[test]
    fn test_insufficient_funds() {
        let money1 = Money::new(dec!(50.00), currencies::usd()).unwrap();
        let money2 = Money::new(dec!(75.00), currencies::usd()).unwrap();
        let result = money1.subtract(&money2);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MoneyError::InsufficientFunds));
    }
}
