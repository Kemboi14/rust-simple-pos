//! Accounting management handlers

use crate::{AppState, ApiResponse};
use crate::services::accounting_service::AccountingService;
use actix_web::{web, HttpResponse, Result};
use tracing::error as err;
use serde::Deserialize;
use sqlx::Row;
use kipko_core::accounting::*;
use chrono::{Utc, TimeZone};
use uuid::Uuid;

/// Get all transactions
pub async fn get_transactions(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let rows = sqlx::query(
        r#"
        SELECT
            id, description, reference_id, posted_at, created_at
        FROM transactions
        ORDER BY posted_at DESC
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        err!("Failed to fetch transactions: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch transactions"}))
    })?;

    let transactions: Vec<Transaction> = rows.into_iter().map(|row| Transaction {
        id: row.get("id"),
        description: row.get("description"),
        reference_id: row.get("reference_id"),
        posted_at: row.get("posted_at"),
        created_at: row.get("created_at"),
        currency: "KES".to_string(),
        exchange_rate: rust_decimal::Decimal::ONE,
        period_id: None,
    }).collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(transactions)))
}

/// Get all accounts
pub async fn get_accounts(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let rows = sqlx::query(
        r#"
        SELECT
            id, name, account_type, description, is_active,
            created_at, updated_at
        FROM accounts
        ORDER BY name
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        err!("Failed to fetch accounts: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch accounts"}))
    })?;

    let accounts: Vec<Account> = rows.into_iter().map(|row| Account {
        id: row.get("id"),
        name: row.get("name"),
        account_type: match row.get::<&str, _>("account_type") {
            "Asset" => AccountType::Asset,
            "Liability" => AccountType::Liability,
            "Equity" => AccountType::Equity,
            "Revenue" => AccountType::Revenue,
            "Expense" => AccountType::Expense,
            _ => AccountType::Asset,
        },
        description: row.get("description"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
        parent_id: None,
        account_code: None,
        normal_balance: None,
        opening_balance: None,
        current_balance: rust_decimal::Decimal::ZERO,
        currency: "KES".to_string(),
        reconciliation_account: false,
    }).collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(accounts)))
}

/// Get account balances
pub async fn get_account_balances(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let accounting_service = AccountingService::new(state.db_pool.clone());
    let balances = accounting_service.get_account_balances().await
        .map_err(|e| {
            err!("Failed to get account balances: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to get account balances"}))
        })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(balances)))
}

/// Get trial balance
#[derive(Debug, Deserialize)]
pub struct TrialBalanceQuery {
    pub as_of_date: Option<String>,
}

pub async fn get_trial_balance(
    state: web::Data<AppState>,
    query: Query<TrialBalanceQuery>,
) -> Result<HttpResponse> {
    let as_of_date = if let Some(date_str) = query.as_of_date {
        date_str.parse::<chrono::DateTime<Utc>>()
            .unwrap_or_else(|_| Utc::now())
    } else {
        Utc::now()
    };

    let accounting_service = AccountingService::new(state.db_pool.clone());
    let trial_balance = accounting_service.calculate_trial_balance(as_of_date).await
        .map_err(|e| {
            err!("Failed to calculate trial balance: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to calculate trial balance"}))
        })?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(trial_balance)))
}

/// Get accounting periods
pub async fn get_accounting_periods(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let rows = sqlx::query(
        r#"
        SELECT
            id, period_name, start_date, end_date, is_closed, closed_at, closed_by, fiscal_year, created_at, updated_at
        FROM accounting_periods
        ORDER BY fiscal_year DESC, start_date DESC
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        err!("Failed to fetch accounting periods: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch accounting periods"}))
    })?;

    let periods: Vec<AccountingPeriod> = rows.into_iter().map(|row| AccountingPeriod {
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
    }).collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(periods)))
}

/// Close accounting period
pub async fn close_accounting_period(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let id = path.into_inner();
    // Get staff ID from context (simplified - in real app, from auth)
    let staff_id = Uuid::new_v4(); // This should come from authentication

    let row = sqlx::query(
        r#"
        UPDATE accounting_periods
        SET is_closed = true, closed_at = CURRENT_TIMESTAMP, closed_by = $2, updated_at = CURRENT_TIMESTAMP
        WHERE id = $1 AND is_closed = false
        RETURNING
            id, period_name, start_date, end_date, is_closed, closed_at, closed_by, fiscal_year, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(staff_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        err!("Failed to close accounting period: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to close accounting period"}))
    })?;

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

    Ok(HttpResponse::Ok().json(ApiResponse::success(period)))
}

/// Generate income statement
pub async fn get_income_statement(
    state: web::Data<AppState>,
    query: Query<TrialBalanceQuery>,
) -> Result<HttpResponse> {
    let as_of_date = if let Some(date_str) = query.as_of_date {
        date_str.parse::<chrono::DateTime<Utc>>()
            .unwrap_or_else(|_| Utc::now())
    } else {
        Utc::now()
    };

    // Calculate income statement from account balances
    let accounting_service = AccountingService::new(state.db_pool.clone());
    let balances = accounting_service.get_account_balances().await
        .map_err(|e| {
            err!("Failed to get account balances for income statement: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to get account balances"}))
        })?;

    let mut income_statement = IncomeStatement::new(None, as_of_date.with_timezone(&Utc).with_month(1).unwrap().with_day(1).unwrap(), as_of_date);

    for balance in &balances {
        match balance.account_type {
            AccountType::Revenue => {
                income_statement.revenue += balance.net_balance;
                income_statement.revenue_breakdown.push((balance.account_name.clone(), balance.net_balance));
            }
            AccountType::Expense => {
                if balance.account_name.contains("Cost") || balance.account_name.contains("Food") || balance.account_name.contains("Beverage") {
                    income_statement.cost_of_goods_sold += balance.net_balance;
                } else {
                    income_statement.operating_expenses += balance.net_balance;
                }
                income_statement.expense_breakdown.push((balance.account_name.clone(), balance.net_balance));
            }
            _ => {}
        }
    }

    income_statement.calculate();

    Ok(HttpResponse::Ok().json(ApiResponse::success(income_statement)))
}

/// Generate balance sheet
pub async fn get_balance_sheet(
    state: web::Data<AppState>,
    query: Query<TrialBalanceQuery>,
) -> Result<HttpResponse> {
    let as_of_date = if let Some(date_str) = query.as_of_date {
        date_str.parse::<chrono::DateTime<Utc>>()
            .unwrap_or_else(|_| Utc::now())
    } else {
        Utc::now()
    };

    let accounting_service = AccountingService::new(state.db_pool.clone());
    let balances = accounting_service.get_account_balances().await
        .map_err(|e| {
            err!("Failed to get account balances for balance sheet: {}", e);
            return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to get account balances"}))
        })?;

    let mut balance_sheet = BalanceSheet::new(None, as_of_date);

    for balance in &balances {
        match balance.account_type {
            AccountType::Asset => {
                balance_sheet.assets += balance.net_balance;
                if balance.account_name.contains("Cash") || balance.account_name.contains("Receivable") || balance.account_name.contains("Inventory") {
                    balance_sheet.current_assets += balance.net_balance;
                } else {
                    balance_sheet.non_current_assets += balance.net_balance;
                }
            }
            AccountType::Liability => {
                balance_sheet.liabilities += balance.net_balance;
                if balance.account_name.contains("Payable") {
                    balance_sheet.current_liabilities += balance.net_balance;
                } else {
                    balance_sheet.non_current_liabilities += balance.net_balance;
                }
            }
            AccountType::Equity => {
                balance_sheet.equity += balance.net_balance;
            }
            _ => {}
        }
    }

    balance_sheet.calculate();

    Ok(HttpResponse::Ok().json(ApiResponse::success(balance_sheet)))
}

/// Bank reconciliation request
#[derive(Debug, Deserialize)]
pub struct BankReconciliationRequest {
    pub account_id: Uuid,
    pub statement_balance: rust_decimal::Decimal,
    pub reconciliation_date: String,
    pub notes: Option<String>,
}

/// Create bank reconciliation
pub async fn create_bank_reconciliation(
    state: web::Data<AppState>,
    request: web::Json<BankReconciliationRequest>,
) -> Result<HttpResponse> {
    // Parse the reconciliation date
    let reconciliation_date = request.reconciliation_date.parse::<chrono::DateTime<Utc>>()
        .map_err(|_| {
            err!("Invalid date format for reconciliation date");
            return HttpResponse::BadRequest().json(serde_json::json!({"error": "Invalid date format"}))
        })?;

    // Get current book balance for the account
    let book_balance_row = sqlx::query(
        r#"
        SELECT current_balance
        FROM accounts
        WHERE id = $1
        "#
    )
    .bind(request.account_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        err!("Failed to get account balance: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to get account balance"}))
    })?;

    let book_balance: rust_decimal::Decimal = book_balance_row.get("current_balance");

    // Create bank reconciliation
    let row = sqlx::query(
        r#"
        INSERT INTO bank_reconciliations (account_id, reconciliation_date, statement_balance, book_balance, difference, status, notes, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
        RETURNING id, account_id, reconciliation_date, statement_balance, book_balance, difference, status, reconciled_by, notes, created_at, updated_at
        "#
    )
    .bind(request.account_id)
    .bind(reconciliation_date)
    .bind(request.statement_balance)
    .bind(book_balance)
    .bind(request.statement_balance - book_balance)
    .bind("Pending")
    .bind(&request.notes)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        err!("Failed to create bank reconciliation: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to create bank reconciliation"}))
    })?;

    let reconciliation = BankReconciliation {
        id: row.get("id"),
        account_id: row.get("account_id"),
        reconciliation_date: row.get("reconciliation_date"),
        statement_balance: row.get("statement_balance"),
        book_balance: row.get("book_balance"),
        difference: row.get("difference"),
        status: row.get("status"),
        reconciled_by: row.get("reconciled_by"),
        notes: row.get("notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(reconciliation)))
}

/// Get bank reconciliations for an account
pub async fn get_bank_reconciliations(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let account_id = path.into_inner();

    let rows = sqlx::query(
        r#"
        SELECT id, account_id, reconciliation_date, statement_balance, book_balance, difference, status, reconciled_by, notes, created_at, updated_at
        FROM bank_reconciliations
        WHERE account_id = $1
        ORDER BY reconciliation_date DESC
        "#
    )
    .bind(account_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        err!("Failed to fetch bank reconciliations: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch bank reconciliations"}))
    })?;

    let reconciliations: Vec<BankReconciliation> = rows.into_iter().map(|row| BankReconciliation {
        id: row.get("id"),
        account_id: row.get("account_id"),
        reconciliation_date: row.get("reconciliation_date"),
        statement_balance: row.get("statement_balance"),
        book_balance: row.get("book_balance"),
        difference: row.get("difference"),
        status: row.get("status"),
        reconciled_by: row.get("reconciled_by"),
        notes: row.get("notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(reconciliations)))
}
