//! Double-entry accounting system for Kipko POS
//! 
//! This module implements a robust double-entry accounting system that ensures
//! financial integrity by requiring every transaction to balance (debits = credits).

use crate::money::Money;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use thiserror::Error;

/// Accounting errors
#[derive(Debug, Error, PartialEq)]
pub enum AccountingError {
    #[error("Transaction does not balance: debits ({debits}) != credits ({credits})")]
    UnbalancedTransaction { debits: Money, credits: Money },
    #[error("Invalid account type for operation")]
    InvalidAccountType,
    #[error("Account not found: {0}")]
    AccountNotFound(Uuid),
    #[error("Duplicate account: {0}")]
    DuplicateAccount(String),
}

/// Result type for accounting operations
pub type AccountingResult<T> = Result<T, AccountingError>;

/// Account types for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

/// Normal balance for account types
impl AccountType {
    pub fn normal_balance(self) -> DebitCredit {
        match self {
            AccountType::Asset | AccountType::Expense => DebitCredit::Debit,
            AccountType::Liability | AccountType::Equity | AccountType::Revenue => DebitCredit::Credit,
        }
    }
}

/// Debit or Credit indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum DebitCredit {
    Debit,
    Credit,
}

/// Account in the chart of accounts
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Account {
    pub id: Uuid,
    pub name: String,
    pub account_type: AccountType,
    pub description: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Account {
    pub fn new(name: String, account_type: AccountType, description: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name,
            account_type,
            description,
            is_active: true,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Journal entry for a single account in a transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct JournalEntry {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub account_id: Uuid,
    pub debit_credit: DebitCredit,
    pub amount: Money,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl JournalEntry {
    pub fn new(
        transaction_id: Uuid,
        account_id: Uuid,
        debit_credit: DebitCredit,
        amount: Money,
        description: Option<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            transaction_id,
            account_id,
            debit_credit,
            amount,
            description,
            created_at: Utc::now(),
        }
    }
}

/// Transaction representing a business event
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Transaction {
    pub id: Uuid,
    pub description: String,
    pub reference_id: Option<Uuid>, // e.g., order_id, payment_id
    pub posted_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl Transaction {
    pub fn new(description: String, reference_id: Option<Uuid>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            description,
            reference_id,
            posted_at: now,
            created_at: now,
        }
    }

    /// Verify that a set of journal entries balances
    pub fn verify_balance(entries: &[JournalEntry]) -> AccountingResult<()> {
        let currency = crate::money::currencies::usd();
        
        let total_debits = entries
            .iter()
            .filter(|entry| matches!(entry.debit_credit, DebitCredit::Debit))
            .fold(Money::zero(currency.clone()), |acc, entry| acc.add(&entry.amount).unwrap());
            
        let total_credits = entries
            .iter()
            .filter(|entry| matches!(entry.debit_credit, DebitCredit::Credit))
            .fold(Money::zero(currency), |acc, entry| acc.add(&entry.amount).unwrap());

        if total_debits == total_credits {
            Ok(())
        } else {
            Err(AccountingError::UnbalancedTransaction {
                debits: total_debits,
                credits: total_credits,
            })
        }
    }
}

/// Chart of Accounts manager
#[derive(Debug, Clone)]
pub struct ChartOfAccounts {
    accounts: Vec<Account>,
}

impl ChartOfAccounts {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
        }
    }

    pub fn add_account(&mut self, account: Account) -> AccountingResult<()> {
        // Check for duplicate names
        if self.accounts.iter().any(|a| a.name == account.name) {
            return Err(AccountingError::DuplicateAccount(account.name));
        }
        
        self.accounts.push(account);
        Ok(())
    }

    pub fn get_account_by_name(&self, name: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.name == name)
    }

    pub fn get_account_by_id(&self, id: &Uuid) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == *id)
    }

    /// Initialize standard restaurant accounts
    pub fn initialize_restaurant_accounts() -> Self {
        let mut coa = Self::new();
        
        // Asset accounts
        coa.add_account(Account::new("Cash".to_string(), AccountType::Asset, Some("Cash on hand".to_string()))).expect("Failed to add Cash account");
        coa.add_account(Account::new("Card Receivable".to_string(), AccountType::Asset, Some("Credit card receivables".to_string()))).expect("Failed to add Card Receivable account");
        coa.add_account(Account::new("Inventory".to_string(), AccountType::Asset, Some("Food and beverage inventory".to_string()))).expect("Failed to add Inventory account");
        
        // Liability accounts
        coa.add_account(Account::new("Tax Payable".to_string(), AccountType::Liability, Some("Sales tax liability".to_string()))).expect("Failed to add Tax Payable account");
        coa.add_account(Account::new("Tips Payable".to_string(), AccountType::Liability, Some("Tips to be distributed".to_string()))).expect("Failed to add Tips Payable account");
        
        // Equity accounts
        coa.add_account(Account::new("Owner's Equity".to_string(), AccountType::Equity, Some("Owner's investment".to_string()))).expect("Failed to add Owner's Equity account");
        
        // Revenue accounts
        coa.add_account(Account::new("Food Revenue".to_string(), AccountType::Revenue, Some("Food sales revenue".to_string()))).expect("Failed to add Food Revenue account");
        coa.add_account(Account::new("Beverage Revenue".to_string(), AccountType::Revenue, Some("Beverage sales revenue".to_string()))).expect("Failed to add Beverage Revenue account");
        coa.add_account(Account::new("Tax Revenue".to_string(), AccountType::Revenue, Some("Sales tax collected".to_string()))).expect("Failed to add Tax Revenue account");
        
        // Expense accounts
        coa.add_account(Account::new("Food Cost".to_string(), AccountType::Expense, Some("Cost of goods sold - food".to_string()))).expect("Failed to add Food Cost account");
        coa.add_account(Account::new("Beverage Cost".to_string(), AccountType::Expense, Some("Cost of goods sold - beverage".to_string()))).expect("Failed to add Beverage Cost account");
        
        coa
    }
}

/// Accounting system that manages transactions and journal entries
#[derive(Debug, Clone)]
pub struct AccountingSystem {
    chart_of_accounts: ChartOfAccounts,
    transactions: Vec<Transaction>,
    journal_entries: Vec<JournalEntry>,
}

impl AccountingSystem {
    pub fn new() -> Self {
        Self {
            chart_of_accounts: ChartOfAccounts::initialize_restaurant_accounts(),
            transactions: Vec::new(),
            journal_entries: Vec::new(),
        }
    }

    /// Record a payment transaction (double-entry)
    pub fn record_payment(
        &mut self,
        payment_amount: Money,
        payment_method: crate::models::PaymentMethod,
        tax_amount: Money,
        order_id: Uuid,
    ) -> AccountingResult<Transaction> {
        let description = format!("Payment for order {} via {:?}", order_id, payment_method);
        let transaction = Transaction::new(description, Some(order_id));
        
        let mut entries = Vec::new();
        
        // Debit cash or card receivable
        let asset_account = match payment_method {
            crate::models::PaymentMethod::Cash => "Cash",
            crate::models::PaymentMethod::Card => "Card Receivable",
            crate::models::PaymentMethod::Mobile => "Card Receivable", // Treat mobile as card
            crate::models::PaymentMethod::GiftCard => "Card Receivable", // Treat gift card as card
        };
        
        let cash_account = self.chart_of_accounts.get_account_by_name(asset_account)
            .ok_or(AccountingError::AccountNotFound(Uuid::nil()))?;
            
        entries.push(JournalEntry::new(
            transaction.id,
            cash_account.id,
            DebitCredit::Debit,
            payment_amount.clone(),
            Some(format!("Payment via {:?}", payment_method)),
        ));
        
        // Credit revenue
        let revenue_account = self.chart_of_accounts.get_account_by_name("Food Revenue")
            .ok_or(AccountingError::AccountNotFound(Uuid::nil()))?;
            
        let net_amount = payment_amount.subtract(&tax_amount).unwrap();
        entries.push(JournalEntry::new(
            transaction.id,
            revenue_account.id,
            DebitCredit::Credit,
            net_amount,
            Some("Food and beverage sales".to_string()),
        ));
        
        // Credit tax payable (if applicable)
        if !tax_amount.is_zero() {
            let tax_account = self.chart_of_accounts.get_account_by_name("Tax Payable")
                .ok_or(AccountingError::AccountNotFound(Uuid::nil()))?;
                
            entries.push(JournalEntry::new(
                transaction.id,
                tax_account.id,
                DebitCredit::Credit,
                tax_amount,
                Some("Sales tax collected".to_string()),
            ));
        }
        
        // Verify the transaction balances
        Transaction::verify_balance(&entries)?;
        
        // Record the transaction and entries
        self.transactions.push(transaction.clone());
        self.journal_entries.extend(entries);
        
        Ok(transaction)
    }

    /// Record a tip transaction
    pub fn record_tip(
        &mut self,
        tip_amount: Money,
        staff_id: Uuid,
        payment_id: Uuid,
    ) -> AccountingResult<Transaction> {
        let description = format!("Tip for staff {}", staff_id);
        let transaction = Transaction::new(description, Some(payment_id));
        
        let mut entries = Vec::new();
        
        // Debit cash (tips reduce cash on hand)
        let cash_account = self.chart_of_accounts.get_account_by_name("Cash")
            .ok_or(AccountingError::AccountNotFound(Uuid::nil()))?;
            
        entries.push(JournalEntry::new(
            transaction.id,
            cash_account.id,
            DebitCredit::Debit,
            tip_amount.clone(),
            Some("Tip distribution".to_string()),
        ));
        
        // Credit tips payable
        let tips_account = self.chart_of_accounts.get_account_by_name("Tips Payable")
            .ok_or(AccountingError::AccountNotFound(Uuid::nil()))?;
            
        entries.push(JournalEntry::new(
            transaction.id,
            tips_account.id,
            DebitCredit::Credit,
            tip_amount,
            Some(format!("Tip for staff {}", staff_id)),
        ));
        
        // Verify the transaction balances
        Transaction::verify_balance(&entries)?;
        
        // Record the transaction and entries
        self.transactions.push(transaction.clone());
        self.journal_entries.extend(entries);
        
        Ok(transaction)
    }

    /// Get account balance
    pub fn get_account_balance(&self, account_name: &str) -> Option<Money> {
        let account = self.chart_of_accounts.get_account_by_name(account_name)?;
        let currency = crate::money::currencies::usd();
        
        let balance = self.journal_entries
            .iter()
            .filter(|entry| entry.account_id == account.id)
            .fold(Money::zero(currency.clone()), |acc, entry| {
                match entry.debit_credit {
                    DebitCredit::Debit => acc.add(&entry.amount).unwrap(),
                    DebitCredit::Credit => acc.subtract(&entry.amount).unwrap_or(acc),
                }
            });
            
        Some(balance)
    }

    /// Get all transactions
    pub fn get_transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    /// Get journal entries for a transaction
    pub fn get_journal_entries(&self, transaction_id: &Uuid) -> Vec<&JournalEntry> {
        self.journal_entries
            .iter()
            .filter(|entry| &entry.transaction_id == transaction_id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::PaymentMethod;
    use rust_decimal_macros::dec;

    #[test]
    fn test_chart_of_accounts_initialization() {
        let coa = ChartOfAccounts::initialize_restaurant_accounts();
        
        // Check that key accounts exist
        assert!(coa.get_account_by_name("Cash").is_some());
        assert!(coa.get_account_by_name("Food Revenue").is_some());
        assert!(coa.get_account_by_name("Tax Payable").is_some());
        assert!(coa.get_account_by_name("Food Cost").is_some());
    }

    #[test]
    fn test_payment_transaction() {
        let mut accounting = AccountingSystem::new();
        
        let payment_amount = Money::new(dec!(100.00), "USD").unwrap();
        let tax_amount = Money::new(dec!(8.50), "USD").unwrap();
        let order_id = Uuid::new_v4();
        
        let transaction = accounting.record_payment(
            payment_amount,
            PaymentMethod::Cash,
            tax_amount,
            order_id,
        ).unwrap();
        
        // Verify transaction was recorded
        assert_eq!(accounting.get_transactions().len(), 1);
        
        // Verify journal entries
        let entries = accounting.get_journal_entries(&transaction.id);
        assert_eq!(entries.len(), 3); // Cash debit, Food Revenue credit, Tax Payable credit
        
        // Verify balances
        let cash_balance = accounting.get_account_balance("Cash").unwrap();
        assert_eq!(cash_balance.amount(), dec!(100.00));
        
        let tax_balance = accounting.get_account_balance("Tax Payable").unwrap();
        assert_eq!(tax_balance.amount(), dec!(8.50));
    }

    #[test]
    fn test_transaction_balance_verification() {
        let transaction_id = Uuid::new_v4();
        let account_id = Uuid::new_v4();
        let amount = Money::new(dec!(100.00), "USD").unwrap();
        
        // Balanced transaction
        let balanced_entries = vec![
            JournalEntry::new(transaction_id, account_id, DebitCredit::Debit, amount, None),
            JournalEntry::new(transaction_id, account_id, DebitCredit::Credit, amount, None),
        ];
        
        assert!(Transaction::verify_balance(&balanced_entries).is_ok());
        
        // Unbalanced transaction
        let unbalanced_entries = vec![
            JournalEntry::new(transaction_id, account_id, DebitCredit::Debit, amount, None),
            JournalEntry::new(transaction_id, account_id, DebitCredit::Credit, 
                Money::new(dec!(50.00), "USD").unwrap(), None),
        ];
        
        assert!(Transaction::verify_balance(&unbalanced_entries).is_err());
    }

    #[test]
    fn test_account_normal_balance() {
        assert_eq!(AccountType::Asset.normal_balance(), DebitCredit::Debit);
        assert_eq!(AccountType::Expense.normal_balance(), DebitCredit::Debit);
        assert_eq!(AccountType::Liability.normal_balance(), DebitCredit::Credit);
        assert_eq!(AccountType::Equity.normal_balance(), DebitCredit::Credit);
        assert_eq!(AccountType::Revenue.normal_balance(), DebitCredit::Credit);
    }
}
