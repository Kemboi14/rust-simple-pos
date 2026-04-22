//! Inventory management handlers

use crate::{AppState, ApiResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use kipko_core::models::*;

/// Inventory transaction creation request
#[derive(Debug, Deserialize)]
pub struct CreateInventoryTransactionRequest {
    pub menu_item_id: Uuid,
    pub transaction_type: InventoryTransactionType,
    pub quantity: i32,
    pub notes: Option<String>,
    pub created_by: Uuid,
}

/// Get all inventory transactions
pub async fn get_inventory_transactions(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<InventoryTransaction>>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT 
            id, menu_item_id, transaction_type, quantity, notes, created_by, created_at
        FROM inventory_transactions
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch inventory transactions: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let transactions: Vec<InventoryTransaction> = rows.into_iter().map(|row| {
        let transaction_type_str: String = row.get("transaction_type");
        let transaction_type = match transaction_type_str.as_str() {
            "StockIn" => InventoryTransactionType::StockIn,
            "StockOut" => InventoryTransactionType::StockOut,
            "Adjustment" => InventoryTransactionType::Adjustment,
            "Transfer" => InventoryTransactionType::Transfer,
            _ => InventoryTransactionType::Adjustment,
        };
        
        InventoryTransaction {
            id: row.get("id"),
            menu_item_id: row.get("menu_item_id"),
            transaction_type,
            quantity: row.get("quantity"),
            notes: row.get("notes"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
        }
    }).collect();

    Ok(Json(ApiResponse::success(transactions)))
}

/// Get inventory transactions for a specific menu item
pub async fn get_inventory_transactions_for_item(
    State(state): State<AppState>,
    Path(menu_item_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<InventoryTransaction>>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT 
            id, menu_item_id, transaction_type, quantity, notes, created_by, created_at
        FROM inventory_transactions
        WHERE menu_item_id = $1
        ORDER BY created_at DESC
        "#
    )
    .bind(menu_item_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch inventory transactions: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let transactions: Vec<InventoryTransaction> = rows.into_iter().map(|row| {
        let transaction_type_str: String = row.get("transaction_type");
        let transaction_type = match transaction_type_str.as_str() {
            "StockIn" => InventoryTransactionType::StockIn,
            "StockOut" => InventoryTransactionType::StockOut,
            "Adjustment" => InventoryTransactionType::Adjustment,
            "Transfer" => InventoryTransactionType::Transfer,
            _ => InventoryTransactionType::Adjustment,
        };
        
        InventoryTransaction {
            id: row.get("id"),
            menu_item_id: row.get("menu_item_id"),
            transaction_type,
            quantity: row.get("quantity"),
            notes: row.get("notes"),
            created_by: row.get("created_by"),
            created_at: row.get("created_at"),
        }
    }).collect();

    Ok(Json(ApiResponse::success(transactions)))
}

/// Create a new inventory transaction
pub async fn create_inventory_transaction(
    State(state): State<AppState>,
    Json(request): Json<CreateInventoryTransactionRequest>,
) -> Result<Json<ApiResponse<InventoryTransaction>>, StatusCode> {
    let transaction_type_str = match request.transaction_type {
        InventoryTransactionType::StockIn => "StockIn",
        InventoryTransactionType::StockOut => "StockOut",
        InventoryTransactionType::Adjustment => "Adjustment",
        InventoryTransactionType::Transfer => "Transfer",
    };

    // Start a transaction
    let mut tx = state.db_pool.begin().await.map_err(|e| {
        tracing::error!("Failed to begin transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Create the inventory transaction
    let row = sqlx::query(
        r#"
        INSERT INTO inventory_transactions (menu_item_id, transaction_type, quantity, notes, created_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, menu_item_id, transaction_type, quantity, notes, created_by, created_at
        "#
    )
    .bind(request.menu_item_id)
    .bind(transaction_type_str)
    .bind(request.quantity)
    .bind(&request.notes)
    .bind(request.created_by)
    .fetch_one(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create inventory transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Update the menu item stock quantity
    let adjustment = match request.transaction_type {
        InventoryTransactionType::StockIn => request.quantity,
        InventoryTransactionType::StockOut => -request.quantity,
        InventoryTransactionType::Adjustment => request.quantity,
        InventoryTransactionType::Transfer => -request.quantity,
    };

    sqlx::query(
        r#"
        UPDATE menu_items
        SET stock_quantity = stock_quantity + $1, updated_at = CURRENT_TIMESTAMP
        WHERE id = $2
        "#
    )
    .bind(adjustment)
    .bind(request.menu_item_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update menu item stock: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // Commit the transaction
    tx.commit().await.map_err(|e| {
        tracing::error!("Failed to commit transaction: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let transaction_type_str: String = row.get("transaction_type");
    let transaction_type = match transaction_type_str.as_str() {
        "StockIn" => InventoryTransactionType::StockIn,
        "StockOut" => InventoryTransactionType::StockOut,
        "Adjustment" => InventoryTransactionType::Adjustment,
        "Transfer" => InventoryTransactionType::Transfer,
        _ => InventoryTransactionType::Adjustment,
    };

    let transaction = InventoryTransaction {
        id: row.get("id"),
        menu_item_id: row.get("menu_item_id"),
        transaction_type,
        quantity: row.get("quantity"),
        notes: row.get("notes"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
    };

    Ok(Json(ApiResponse::success(transaction)))
}
