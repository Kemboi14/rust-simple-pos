//! Kipko POS Core Library
//! 
//! This library contains the core domain models and business logic for the Kipko POS system.
//! It provides high-integrity financial operations using double-entry accounting principles
//! and precise decimal arithmetic for all monetary calculations.

pub mod money;
pub mod models;
pub mod accounting;
pub mod tax;

pub use money::Money;
pub use models::*;
pub use accounting::{AccountingSystem, Transaction, JournalEntry};
