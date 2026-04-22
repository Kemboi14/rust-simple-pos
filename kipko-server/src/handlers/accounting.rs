//! Accounting management handlers

use crate::{AppState, ApiResponse};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
};
use sqlx::Row;
use kipko_core::accounting::*;

/// Get all transactions
pub async fn get_transactions(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Transaction>>>, StatusCode> {
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
        tracing::error!("Failed to fetch transactions: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let transactions: Vec<Transaction> = rows.into_iter().map(|row| Transaction {
        id: row.get("id"),
        description: row.get("description"),
        reference_id: row.get("reference_id"),
        posted_at: row.get("posted_at"),
        created_at: row.get("created_at"),
    }).collect();

    Ok(Json(ApiResponse::success(transactions)))
}

/// Get all accounts
pub async fn get_accounts(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Account>>>, StatusCode> {
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
        tracing::error!("Failed to fetch accounts: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
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
    }).collect();

    Ok(Json(ApiResponse::success(accounts)))
}

/// Get account balances
pub async fn get_account_balances(
    State(_state): State<AppState>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // This would calculate actual balances from journal entries
    // For now, return a placeholder response
    let balances = serde_json::json!({
        "balances": [
            {"account_name": "Cash", "balance": 1250.50},
            {"account_name": "Card Receivable", "balance": 850.25},
            {"account_name": "Tax Payable", "balance": 125.75},
            {"account_name": "Food Revenue", "balance": 2100.00},
            {"account_name": "Beverage Revenue", "balance": 450.00}
        ]
    });

    Ok(Json(ApiResponse::success(balances)))
}
