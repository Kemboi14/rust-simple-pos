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
    pub parent_id: Option<Uuid>,
    pub account_code: Option<String>,
    pub normal_balance: Option<String>,
    pub opening_balance: Option<rust_decimal::Decimal>,
    pub current_balance: rust_decimal::Decimal,
    pub currency: String,
    pub reconciliation_account: bool,
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
            parent_id: None,
            account_code: None,
            normal_balance: Some(if matches!(account_type, AccountType::Asset | AccountType::Expense) { "Debit".to_string() } else { "Credit".to_string() }),
            opening_balance: Some(rust_decimal::Decimal::ZERO),
            current_balance: rust_decimal::Decimal::ZERO,
            currency: "KES".to_string(),
            reconciliation_account: false,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn with_code(mut self, code: String) -> Self {
        self.account_code = Some(code);
        self
    }

    pub fn with_parent(mut self, parent_id: Uuid) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    pub fn set_reconciliation_account(mut self) -> Self {
        self.reconciliation_account = true;
        self
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
    pub period_id: Option<Uuid>,
    pub reference_type: Option<String>,
    pub reference_id: Option<Uuid>,
    pub is_reconciled: bool,
    pub reconciled_date: Option<DateTime<Utc>>,
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
            period_id: None,
            reference_type: None,
            reference_id: None,
            is_reconciled: false,
            reconciled_date: None,
        }
    }

    pub fn with_period(mut self, period_id: Uuid) -> Self {
        self.period_id = Some(period_id);
        self
    }

    pub fn with_reference(mut self, ref_type: String, ref_id: Uuid) -> Self {
        self.reference_type = Some(ref_type);
        self.reference_id = Some(ref_id);
        self
    }

    pub fn mark_reconciled(mut self, date: DateTime<Utc>) -> Self {
        self.is_reconciled = true;
        self.reconciled_date = Some(date);
        self
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
    pub currency: String,
    pub exchange_rate: rust_decimal::Decimal,
    pub period_id: Option<Uuid>,
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
            currency: "KES".to_string(),
            exchange_rate: rust_decimal::Decimal::ONE,
            period_id: None,
        }
    }

    pub fn with_period(mut self, period_id: Uuid) -> Self {
        self.period_id = Some(period_id);
        self
    }

    pub fn with_currency(mut self, currency: String, exchange_rate: rust_decimal::Decimal) -> Self {
        self.currency = currency;
        self.exchange_rate = exchange_rate;
        self
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

        // Asset accounts (1000 series)
        coa.add_account(Account::new("Cash".to_string(), AccountType::Asset, Some("Cash on hand and in registers".to_string()))
            .with_code("1000".to_string())
            .set_reconciliation_account()).expect("Failed to add Cash account");
        coa.add_account(Account::new("Card Receivable".to_string(), AccountType::Asset, Some("Credit card receivables".to_string()))
            .with_code("1100".to_string())
            .set_reconciliation_account()).expect("Failed to add Card Receivable account");
        coa.add_account(Account::new("Mobile Money Receivable".to_string(), AccountType::Asset, Some("M-Pesa and mobile money receivables".to_string()))
            .with_code("1150".to_string())
            .set_reconciliation_account()).expect("Failed to add Mobile Money Receivable account");
        coa.add_account(Account::new("Inventory".to_string(), AccountType::Asset, Some("Food and beverage inventory".to_string()))
            .with_code("1200".to_string())).expect("Failed to add Inventory account");
        coa.add_account(Account::new("Accounts Receivable".to_string(), AccountType::Asset, Some("Customer accounts receivable".to_string()))
            .with_code("1300".to_string())).expect("Failed to add Accounts Receivable account");
        coa.add_account(Account::new("Prepaid Expenses".to_string(), AccountType::Asset, Some("Prepaid expenses".to_string()))
            .with_code("1400".to_string())).expect("Failed to add Prepaid Expenses account");

        // Liability accounts (2000 series)
        coa.add_account(Account::new("Accounts Payable".to_string(), AccountType::Liability, Some("Accounts payable to suppliers".to_string()))
            .with_code("2000".to_string())).expect("Failed to add Accounts Payable account");
        coa.add_account(Account::new("Tax Payable".to_string(), AccountType::Liability, Some("Sales tax liability".to_string()))
            .with_code("2100".to_string())).expect("Failed to add Tax Payable account");
        coa.add_account(Account::new("Tips Payable".to_string(), AccountType::Liability, Some("Tips to be distributed to staff".to_string()))
            .with_code("2200".to_string())).expect("Failed to add Tips Payable account");
        coa.add_account(Account::new("Accrued Expenses".to_string(), AccountType::Liability, Some("Accrued expenses".to_string()))
            .with_code("2300".to_string())).expect("Failed to add Accrued Expenses account");
        coa.add_account(Account::new("Unearned Revenue".to_string(), AccountType::Liability, Some("Deposits and advance payments".to_string()))
            .with_code("2400".to_string())).expect("Failed to add Unearned Revenue account");

        // Equity accounts (3000 series)
        coa.add_account(Account::new("Owner's Equity".to_string(), AccountType::Equity, Some("Owner's investment and retained earnings".to_string()))
            .with_code("3000".to_string())).expect("Failed to add Owner's Equity account");
        coa.add_account(Account::new("Retained Earnings".to_string(), AccountType::Equity, Some("Accumulated retained earnings".to_string()))
            .with_code("3100".to_string())).expect("Failed to add Retained Earnings account");
        coa.add_account(Account::new("Current Period Earnings".to_string(), AccountType::Equity, Some("Current period earnings".to_string()))
            .with_code("3200".to_string())).expect("Failed to add Current Period Earnings account");

        // Revenue accounts (4000 series)
        coa.add_account(Account::new("Food Revenue".to_string(), AccountType::Revenue, Some("Food sales revenue".to_string()))
            .with_code("4000".to_string())).expect("Failed to add Food Revenue account");
        coa.add_account(Account::new("Beverage Revenue".to_string(), AccountType::Revenue, Some("Beverage sales revenue".to_string()))
            .with_code("4100".to_string())).expect("Failed to add Beverage Revenue account");
        coa.add_account(Account::new("Other Revenue".to_string(), AccountType::Revenue, Some("Other revenue sources".to_string()))
            .with_code("4200".to_string())).expect("Failed to add Other Revenue account");
        coa.add_account(Account::new("Service Charges".to_string(), AccountType::Revenue, Some("Service charges and fees".to_string()))
            .with_code("4300".to_string())).expect("Failed to add Service Charges account");

        // Cost of Goods Sold accounts (5000 series)
        coa.add_account(Account::new("Food Cost".to_string(), AccountType::Expense, Some("Cost of goods sold - food".to_string()))
            .with_code("5000".to_string())).expect("Failed to add Food Cost account");
        coa.add_account(Account::new("Beverage Cost".to_string(), AccountType::Expense, Some("Cost of goods sold - beverage".to_string()))
            .with_code("5100".to_string())).expect("Failed to add Beverage Cost account");
        coa.add_account(Account::new("Supply Cost".to_string(), AccountType::Expense, Some("Cost of supplies".to_string()))
            .with_code("5200".to_string())).expect("Failed to add Supply Cost account");

        // Operating Expense accounts (6000 series)
        coa.add_account(Account::new("Labor Cost".to_string(), AccountType::Expense, Some("Staff wages and salaries".to_string()))
            .with_code("6000".to_string())).expect("Failed to add Labor Cost account");
        coa.add_account(Account::new("Rent Expense".to_string(), AccountType::Expense, Some("Rent and lease payments".to_string()))
            .with_code("6100".to_string())).expect("Failed to add Rent Expense account");
        coa.add_account(Account::new("Utilities Expense".to_string(), AccountType::Expense, Some("Utilities (electricity, water, gas)".to_string()))
            .with_code("6200".to_string())).expect("Failed to add Utilities Expense account");
        coa.add_account(Account::new("Marketing Expense".to_string(), AccountType::Expense, Some("Marketing and advertising".to_string()))
            .with_code("6300".to_string())).expect("Failed to add Marketing Expense account");
        coa.add_account(Account::new("Maintenance Expense".to_string(), AccountType::Expense, Some("Equipment and facility maintenance".to_string()))
            .with_code("6400".to_string())).expect("Failed to add Maintenance Expense account");
        coa.add_account(Account::new("License Expense".to_string(), AccountType::Expense, Some("Business licenses and permits".to_string()))
            .with_code("6500".to_string())).expect("Failed to add License Expense account");
        coa.add_account(Account::new("Other Operating Expense".to_string(), AccountType::Expense, Some("Other operating expenses".to_string()))
            .with_code("6600".to_string())).expect("Failed to add Other Operating Expense account");

        coa
    }
}

/// Accounting period for closing and reporting
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct AccountingPeriod {
    pub id: Uuid,
    pub period_name: String,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub is_closed: bool,
    pub closed_at: Option<DateTime<Utc>>,
    pub closed_by: Option<Uuid>,
    pub fiscal_year: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AccountingPeriod {
    pub fn new(period_name: String, start_date: DateTime<Utc>, end_date: DateTime<Utc>, fiscal_year: i32) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            period_name,
            start_date,
            end_date,
            is_closed: false,
            closed_at: None,
            closed_by: None,
            fiscal_year,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn close(&mut self, closed_by: Uuid) {
        self.is_closed = true;
        self.closed_at = Some(Utc::now());
        self.closed_by = Some(closed_by);
        self.updated_at = Utc::now();
    }
}

/// Bank reconciliation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct BankReconciliation {
    pub id: Uuid,
    pub account_id: Uuid,
    pub reconciliation_date: DateTime<Utc>,
    pub statement_balance: rust_decimal::Decimal,
    pub book_balance: rust_decimal::Decimal,
    pub difference: rust_decimal::Decimal,
    pub status: String,
    pub reconciled_by: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl BankReconciliation {
    pub fn new(account_id: Uuid, reconciliation_date: DateTime<Utc>, statement_balance: rust_decimal::Decimal, book_balance: rust_decimal::Decimal) -> Self {
        let now = Utc::now();
        let difference = statement_balance - book_balance;
        Self {
            id: Uuid::new_v4(),
            account_id,
            reconciliation_date,
            statement_balance,
            book_balance,
            difference,
            status: if difference == rust_decimal::Decimal::ZERO { "Completed".to_string() } else { "Pending".to_string() },
            reconciled_by: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn complete(&mut self, reconciled_by: Uuid) {
        self.status = "Completed".to_string();
        self.reconciled_by = Some(reconciled_by);
        self.updated_at = Utc::now();
    }
}

/// Reconciliation item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct ReconciliationItem {
    pub id: Uuid,
    pub reconciliation_id: Uuid,
    pub journal_entry_id: Option<Uuid>,
    pub item_type: String,
    pub amount: rust_decimal::Decimal,
    pub description: Option<String>,
    pub is_cleared: bool,
    pub cleared_date: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl ReconciliationItem {
    pub fn new(reconciliation_id: Uuid, item_type: String, amount: rust_decimal::Decimal, description: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            reconciliation_id,
            journal_entry_id: None,
            item_type,
            amount,
            description,
            is_cleared: false,
            cleared_date: None,
            created_at: Utc::now(),
        }
    }

    pub fn clear(&mut self) {
        self.is_cleared = true;
        self.cleared_date = Some(Utc::now());
    }

    pub fn with_journal_entry(mut self, entry_id: Uuid) -> Self {
        self.journal_entry_id = Some(entry_id);
        self
    }
}

/// Financial report
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct FinancialReport {
    pub id: Uuid,
    pub report_type: String,
    pub period_id: Option<Uuid>,
    pub report_data: serde_json::Value,
    pub generated_by: Uuid,
    pub generated_at: DateTime<Utc>,
    pub parameters: Option<serde_json::Value>,
}

impl FinancialReport {
    pub fn new(report_type: String, period_id: Option<Uuid>, report_data: serde_json::Value, generated_by: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            report_type,
            period_id,
            report_data,
            generated_by,
            generated_at: Utc::now(),
            parameters: None,
        }
    }

    pub fn with_parameters(mut self, parameters: serde_json::Value) -> Self {
        self.parameters = Some(parameters);
        self
    }
}

/// Audit log entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct AuditLogEntry {
    pub id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action_type: String,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub changed_by: Option<Uuid>,
    pub changed_at: DateTime<Utc>,
    pub description: Option<String>,
}

/// Account balance information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountBalance {
    pub account_id: Uuid,
    pub account_name: String,
    pub account_type: AccountType,
    pub debit_balance: rust_decimal::Decimal,
    pub credit_balance: rust_decimal::Decimal,
    pub net_balance: rust_decimal::Decimal,
}

impl AccountBalance {
    pub fn new(account: &Account, debit_balance: rust_decimal::Decimal, credit_balance: rust_decimal::Decimal) -> Self {
        let net_balance = if matches!(account.account_type, AccountType::Asset | AccountType::Expense) {
            debit_balance - credit_balance
        } else {
            credit_balance - debit_balance
        };

        Self {
            account_id: account.id,
            account_name: account.name.clone(),
            account_type: account.account_type,
            debit_balance,
            credit_balance,
            net_balance,
        }
    }
}

/// Trial balance report
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrialBalance {
    pub period_id: Option<Uuid>,
    pub as_of_date: DateTime<Utc>,
    pub accounts: Vec<AccountBalance>,
    pub total_debits: rust_decimal::Decimal,
    pub total_credits: rust_decimal::Decimal,
    pub is_balanced: bool,
}

impl TrialBalance {
    pub fn new(period_id: Option<Uuid>, as_of_date: DateTime<Utc>) -> Self {
        Self {
            period_id,
            as_of_date,
            accounts: Vec::new(),
            total_debits: rust_decimal::Decimal::ZERO,
            total_credits: rust_decimal::Decimal::ZERO,
            is_balanced: true,
        }
    }

    pub fn add_account(&mut self, balance: AccountBalance) {
        self.total_debits += balance.debit_balance;
        self.total_credits += balance.credit_balance;
        self.accounts.push(balance);
        self.is_balanced = self.total_debits == self.total_credits;
    }
}

/// Income statement (P&L) data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IncomeStatement {
    pub period_id: Option<Uuid>,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub revenue: rust_decimal::Decimal,
    pub cost_of_goods_sold: rust_decimal::Decimal,
    pub gross_profit: rust_decimal::Decimal,
    pub operating_expenses: rust_decimal::Decimal,
    pub net_income: rust_decimal::Decimal,
    pub revenue_breakdown: Vec<(String, rust_decimal::Decimal)>,
    pub expense_breakdown: Vec<(String, rust_decimal::Decimal)>,
}

impl IncomeStatement {
    pub fn new(period_id: Option<Uuid>, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Self {
        Self {
            period_id,
            start_date,
            end_date,
            revenue: rust_decimal::Decimal::ZERO,
            cost_of_goods_sold: rust_decimal::Decimal::ZERO,
            gross_profit: rust_decimal::Decimal::ZERO,
            operating_expenses: rust_decimal::Decimal::ZERO,
            net_income: rust_decimal::Decimal::ZERO,
            revenue_breakdown: Vec::new(),
            expense_breakdown: Vec::new(),
        }
    }

    pub fn calculate(&mut self) {
        self.gross_profit = self.revenue - self.cost_of_goods_sold;
        self.net_income = self.gross_profit - self.operating_expenses;
    }
}

/// Balance sheet data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BalanceSheet {
    pub period_id: Option<Uuid>,
    pub as_of_date: DateTime<Utc>,
    pub assets: rust_decimal::Decimal,
    pub current_assets: rust_decimal::Decimal,
    pub non_current_assets: rust_decimal::Decimal,
    pub liabilities: rust_decimal::Decimal,
    pub current_liabilities: rust_decimal::Decimal,
    pub non_current_liabilities: rust_decimal::Decimal,
    pub equity: rust_decimal::Decimal,
    pub total_liabilities_equity: rust_decimal::Decimal,
    pub is_balanced: bool,
}

impl BalanceSheet {
    pub fn new(period_id: Option<Uuid>, as_of_date: DateTime<Utc>) -> Self {
        Self {
            period_id,
            as_of_date,
            assets: rust_decimal::Decimal::ZERO,
            current_assets: rust_decimal::Decimal::ZERO,
            non_current_assets: rust_decimal::Decimal::ZERO,
            liabilities: rust_decimal::Decimal::ZERO,
            current_liabilities: rust_decimal::Decimal::ZERO,
            non_current_liabilities: rust_decimal::Decimal::ZERO,
            equity: rust_decimal::Decimal::ZERO,
            total_liabilities_equity: rust_decimal::Decimal::ZERO,
            is_balanced: true,
        }
    }

    pub fn calculate(&mut self) {
        self.total_liabilities_equity = self.liabilities + self.equity;
        self.is_balanced = self.assets == self.total_liabilities_equity;
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
            crate::models::PaymentMethod::MobileMoney => "Card Receivable", // Treat mobile money as card
            crate::models::PaymentMethod::Mpesa => "Card Receivable", // Treat M-Pesa as card
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
