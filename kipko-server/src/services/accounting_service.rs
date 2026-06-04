//! Accounting Service
//!
//! This service handles automatic accounting integration with payments, orders,
//! and other business transactions to ensure proper double-entry bookkeeping.

use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, DateTime};
use anyhow::Result;
use kipko_core::accounting::*;
use kipko_core::money::{Money, currencies};

pub struct AccountingService {
    pool: PgPool,
}

impl AccountingService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Get account by name
    async fn get_account_by_name(&self, name: &str) -> Result<Option<Account>> {
        let row = sqlx::query(
            r#"
            SELECT id, name, account_type, description, is_active, parent_id, account_code,
                   normal_balance, opening_balance, current_balance, currency, reconciliation_account,
                   created_at, updated_at
            FROM accounts
            WHERE name = $1 AND is_active = true
            "#
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let account_type_str: String = row.get("account_type");
            let account_type = match account_type_str.as_str() {
                "Asset" => AccountType::Asset,
                "Liability" => AccountType::Liability,
                "Equity" => AccountType::Equity,
                "Revenue" => AccountType::Revenue,
                "Expense" => AccountType::Expense,
                _ => return Err(anyhow::anyhow!("Unknown account type: {}", account_type_str)),
            };

            let account = Account {
                id: row.get("id"),
                name: row.get("name"),
                account_type,
                description: row.get("description"),
                is_active: row.get("is_active"),
                parent_id: row.get("parent_id"),
                account_code: row.get("account_code"),
                normal_balance: row.get("normal_balance"),
                opening_balance: row.get("opening_balance"),
                current_balance: row.get("current_balance"),
                currency: row.get("currency"),
                reconciliation_account: row.get("reconciliation_account"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(account))
        } else {
            Ok(None)
        }
    }

    /// Get current accounting period for a given date
    async fn get_period_for_date(&self, date: DateTime<Utc>) -> Result<Option<AccountingPeriod>> {
        let row = sqlx::query(
            r#"
            SELECT id, period_name, start_date, end_date, is_closed, closed_at, closed_by, fiscal_year, created_at, updated_at
            FROM accounting_periods
            WHERE start_date <= $1 AND end_date >= $1 AND is_closed = false
            ORDER BY start_date DESC
            LIMIT 1
            "#
        )
        .bind(date)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let period = AccountingPeriod {
                id: row.get("id"),
                period_name: row.get("period_name"),
                start_date: row.get("start_date"),
                end_date: row.get("end_date"),
                is_closed: row.get("is_closed"),
                closed_at: row.get("closed_at"),
                closed_by: row.get("closed_by"),
                fiscal_year: row.get("fiscal_year"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Some(period))
        } else {
            Ok(None)
        }
    }

    /// Create a transaction with journal entries
    async fn create_transaction(&self, description: String, reference_id: Option<Uuid>) -> Result<Uuid> {
        let currency = currencies::ksh();

        let row = sqlx::query(
            r#"
            INSERT INTO transactions (description, reference_id, currency, exchange_rate, posted_at, created_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id
            "#
        )
        .bind(&description)
        .bind(reference_id)
        .bind("KES")
        .bind(1.0)
        .bind(Utc::now())
        .bind(Utc::now())
        .fetch_one(&self.pool)
        .await?;

        Ok(row.get("id"))
    }

    /// Create journal entries for a transaction
    async fn create_journal_entries(&self, transaction_id: Uuid, entries: Vec<JournalEntry>) -> Result<()> {
        for entry in entries {
            let debit_credit_str = match entry.debit_credit {
                DebitCredit::Debit => "Debit",
                DebitCredit::Credit => "Credit",
            };

            sqlx::query(
                r#"
                INSERT INTO journal_entries (transaction_id, account_id, debit_credit, amount, description, created_at, period_id, reference_type, reference_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#
            )
            .bind(transaction_id)
            .bind(entry.account_id)
            .bind(debit_credit_str)
            .bind(entry.amount.amount())
            .bind(&entry.description)
            .bind(Utc::now())
            .bind(entry.period_id)
            .bind(&entry.reference_type)
            .bind(entry.reference_id)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    /// Create accounting entries for a payment
    pub async fn create_payment_accounting_entries(
        &self,
        payment_id: Uuid,
        amount: rust_decimal::Decimal,
        payment_method: &str,
    ) -> Result<()> {
        // Get appropriate accounts based on payment method
        let (cash_account, revenue_account) = match payment_method {
            "Cash" => ("Cash", "Food Revenue"),
            "Card" => ("Card Receivable", "Food Revenue"),
            "MobileMoney" | "Mpesa" => ("Mobile Money Receivable", "Food Revenue"),
            _ => ("Cash", "Food Revenue"),
        };

        let cash_acc = self.get_account_by_name(cash_account).await?
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", cash_account))?;
        let revenue_acc = self.get_account_by_name(revenue_account).await?
            .ok_or_else(|| anyhow::anyhow!("Account not found: {}", revenue_account))?;

        // Get current period
        let period = self.get_period_for_date(Utc::now()).await?;

        // Create transaction
        let description = format!("Payment received - {}", payment_method);
        let transaction_id = self.create_transaction(description, Some(payment_id)).await?;

        // Create journal entries (double entry)
        let currency = currencies::ksh();
        let amount_money = Money::new(amount, currency.clone());

        let journal_entries = vec![
            JournalEntry::new(transaction_id, cash_acc.id, DebitCredit::Debit, amount_money.clone(), Some("Cash receipt".to_string()))
                .with_period(period.as_ref().map(|p| p.id).unwrap_or_else(|| Uuid::new_v4()))
                .with_reference("Payment".to_string(), payment_id),
            JournalEntry::new(transaction_id, revenue_acc.id, DebitCredit::Credit, amount_money, Some("Revenue recognition".to_string()))
                .with_period(period.as_ref().map(|p| p.id).unwrap_or_else(|| Uuid::new_v4()))
                .with_reference("Payment".to_string(), payment_id),
        ];

        self.create_journal_entries(transaction_id, journal_entries).await?;

        Ok(())
    }

    /// Create accounting entries for tax collection
    pub async fn create_tax_accounting_entries(
        &self,
        order_id: Uuid,
        tax_amount: rust_decimal::Decimal,
    ) -> Result<()> {
        let tax_payable_account = self.get_account_by_name("Tax Payable").await?
            .ok_or_else(|| anyhow::anyhow!("Account not found: Tax Payable"))?;
        let tax_revenue_account = self.get_account_by_name("Other Revenue").await?
            .ok_or_else(|| anyhow::anyhow!("Account not found: Other Revenue"))?;

        // Get current period
        let period = self.get_period_for_date(Utc::now()).await?;

        // Create transaction
        let description = "Sales tax collected".to_string();
        let transaction_id = self.create_transaction(description, Some(order_id)).await?;

        // Create journal entries
        let currency = currencies::ksh();
        let tax_money = Money::new(tax_amount, currency.clone());

        let journal_entries = vec![
            JournalEntry::new(transaction_id, tax_payable_account.id, DebitCredit::Debit, tax_money.clone(), Some("Tax liability".to_string()))
                .with_period(period.as_ref().map(|p| p.id).unwrap_or_else(|| Uuid::new_v4()))
                .with_reference("Order".to_string(), order_id),
            JournalEntry::new(transaction_id, tax_revenue_account.id, DebitCredit::Credit, tax_money, Some("Tax collected".to_string()))
                .with_period(period.as_ref().map(|p| p.id).unwrap_or_else(|| Uuid::new_v4()))
                .with_reference("Order".to_string(), order_id),
        ];

        self.create_journal_entries(transaction_id, journal_entries).await?;

        Ok(())
    }

    /// Create accounting entries for tips
    pub async fn create_tips_accounting_entries(
        &self,
        payment_id: Uuid,
        tip_amount: rust_decimal::Decimal,
    ) -> Result<()> {
        if tip_amount <= rust_decimal::Decimal::ZERO {
            return Ok(());
        }

        let tips_payable_account = self.get_account_by_name("Tips Payable").await?
            .ok_or_else(|| anyhow::anyhow!("Account not found: Tips Payable"))?;

        // Get current period
        let period = self.get_period_for_date(Utc::now()).await?;

        // Create transaction
        let description = "Tips collected".to_string();
        let transaction_id = self.create_transaction(description, Some(payment_id)).await?;

        // Create journal entries
        let currency = currencies::ksh();
        let tips_money = Money::new(tip_amount, currency.clone());

        let journal_entries = vec![
            JournalEntry::new(transaction_id, tips_payable_account.id, DebitCredit::Debit, tips_money, Some("Tips liability".to_string()))
                .with_period(period.as_ref().map(|p| p.id).unwrap_or_else(|| Uuid::new_v4()))
                .with_reference("Payment".to_string(), payment_id),
            JournalEntry::new(transaction_id, tips_payable_account.id, DebitCredit::Credit, tips_money, Some("Tips to distribute".to_string()))
                .with_period(period.as_ref().map(|p| p.id).unwrap_or_else(|| Uuid::new_v4()))
                .with_reference("Payment".to_string(), payment_id),
        ];

        self.create_journal_entries(transaction_id, journal_entries).await?;

        Ok(())
    }

    /// Calculate trial balance as of a specific date
    pub async fn calculate_trial_balance(&self, as_of_date: DateTime<Utc>) -> Result<TrialBalance> {
        let rows = sqlx::query(
            r#"
            SELECT
                a.id,
                a.name,
                a.account_type,
                a.current_balance,
                COALESCE(SUM(CASE WHEN je.debit_credit = 'Debit' THEN je.amount ELSE 0 END), 0) as total_debits,
                COALESCE(SUM(CASE WHEN je.debit_credit = 'Credit' THEN je.amount ELSE 0 END), 0) as total_credits
            FROM accounts a
            LEFT JOIN journal_entries je ON a.id = je.account_id
            WHERE a.is_active = true
            GROUP BY a.id, a.name, a.account_type, a.current_balance
            ORDER BY a.account_code
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut trial_balance = TrialBalance::new(None, as_of_date);

        for row in rows {
            let account_type_str: String = row.get("account_type");
            let account_type = match account_type_str.as_str() {
                "Asset" => AccountType::Asset,
                "Liability" => AccountType::Liability,
                "Equity" => AccountType::Equity,
                "Revenue" => AccountType::Revenue,
                "Expense" => AccountType::Expense,
                _ => continue,
            };

            let account = Account {
                id: row.get("id"),
                name: row.get("name"),
                account_type,
                description: None,
                is_active: true,
                parent_id: None,
                account_code: None,
                normal_balance: None,
                opening_balance: None,
                current_balance: row.get("current_balance"),
                currency: "KES".to_string(),
                reconciliation_account: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let debit_balance: rust_decimal::Decimal = row.get("total_debits");
            let credit_balance: rust_decimal::Decimal = row.get("total_credits");

            let balance = AccountBalance::new(&account, debit_balance, credit_balance);
            trial_balance.add_account(balance);
        }

        Ok(trial_balance)
    }

    /// Get account balances for financial statements
    pub async fn get_account_balances(&self) -> Result<Vec<AccountBalance>> {
        let rows = sqlx::query(
            r#"
            SELECT
                a.id,
                a.name,
                a.account_type,
                a.current_balance,
                COALESCE(SUM(CASE WHEN je.debit_credit = 'Debit' THEN je.amount ELSE 0 END), 0) as total_debits,
                COALESCE(SUM(CASE WHEN je.debit_credit = 'Credit' THEN je.amount ELSE 0 END), 0) as total_credits
            FROM accounts a
            LEFT JOIN journal_entries je ON a.id = je.account_id
            WHERE a.is_active = true
            GROUP BY a.id, a.name, a.account_type, a.current_balance
            ORDER BY a.account_code
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut balances = Vec::new();

        for row in rows {
            let account_type_str: String = row.get("account_type");
            let account_type = match account_type_str.as_str() {
                "Asset" => AccountType::Asset,
                "Liability" => AccountType::Liability,
                "Equity" => AccountType::Equity,
                "Revenue" => AccountType::Revenue,
                "Expense" => AccountType::Expense,
                _ => continue,
            };

            let account = Account {
                id: row.get("id"),
                name: row.get("name"),
                account_type,
                description: None,
                is_active: true,
                parent_id: None,
                account_code: None,
                normal_balance: None,
                opening_balance: None,
                current_balance: row.get("current_balance"),
                currency: "KES".to_string(),
                reconciliation_account: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let debit_balance: rust_decimal::Decimal = row.get("total_debits");
            let credit_balance: rust_decimal::Decimal = row.get("total_credits");

            let balance = AccountBalance::new(&account, debit_balance, credit_balance);
            balances.push(balance);
        }

        Ok(balances)
    }
}