//! Order items management handlers

use crate::{AppState, ApiResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use kipko_core::{OrderItem, MenuItem, OrderItemStatus};

#[derive(Deserialize)]
pub struct AddOrderItemRequest {
    pub menu_item_id: Uuid,
    pub quantity: i32,
    pub notes: Option<String>,
}

#[derive(serde::Deserialize)]
pub struct UpdateOrderItemRequest {
    pub quantity: Option<i32>,
    pub notes: Option<String>,
    pub preparation_status: Option<String>,
}

pub async fn get_order_items(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<OrderItem>>>, axum::http::StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT id, order_id, menu_item_id, quantity, unit_price, status, notes, void_reason, void_by, created_at, updated_at
        FROM order_items
        WHERE order_id = $1
        ORDER BY created_at
        "#
    )
    .bind(order_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch order items: {}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let items: Vec<OrderItem> = rows
        .into_iter()
        .map(|row| {
            let menu_item_id: Uuid = row.get("menu_item_id");
            let quantity: i32 = row.get("quantity");
            let unit_price: rust_decimal::Decimal = row.get("unit_price");
            let currency = kipko_core::money::currencies::ksh();

            OrderItem {
                id: row.get("id"),
                order_id: row.get("order_id"),
                menu_item_id,
                quantity,
                unit_price: kipko_core::Money::new(unit_price, currency).unwrap(),
                status: match row.get::<&str, _>("status") {
                    "Pending" => OrderItemStatus::Pending,
                    "Fired" => OrderItemStatus::Fired,
                    "Ready" => OrderItemStatus::Ready,
                    "Delivered" => OrderItemStatus::Delivered,
                    "Voided" => OrderItemStatus::Voided,
                    _ => OrderItemStatus::Pending,
                },
                notes: row.get("notes"),
                void_reason: row.get("void_reason"),
                void_by: row.get("void_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect();

    Ok(Json(ApiResponse::success(items)))
}

pub async fn add_order_item(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
    Json(request): Json<AddOrderItemRequest>,
) -> Result<Json<ApiResponse<OrderItem>>, axum::http::StatusCode> {
    // Get menu item price
    let menu_item_row = sqlx::query(
        "SELECT price FROM menu_items WHERE id = $1"
    )
    .bind(request.menu_item_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch menu item: {}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or_else(|| {
        tracing::error!("Menu item not found");
        axum::http::StatusCode::NOT_FOUND
    })?;

    let price: rust_decimal::Decimal = menu_item_row.get("price");

    let row = sqlx::query(
        r#"
        INSERT INTO order_items (order_id, menu_item_id, quantity, unit_price, notes, status)
        VALUES ($1, $2, $3, $4, $5, 'Pending')
        RETURNING id, order_id, menu_item_id, quantity, unit_price, status, notes, void_reason, void_by, created_at, updated_at
        "#
    )
    .bind(order_id)
    .bind(request.menu_item_id)
    .bind(request.quantity)
    .bind(price)
    .bind(&request.notes)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to add order item: {}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let currency = kipko_core::money::currencies::ksh();
    let item = OrderItem {
        id: row.get("id"),
        order_id: row.get("order_id"),
        menu_item_id: row.get("menu_item_id"),
        quantity: row.get("quantity"),
        unit_price: kipko_core::Money::new(row.get("price"), currency).unwrap(),
        status: OrderItemStatus::Pending,
        notes: row.get("notes"),
        void_reason: None,
        void_by: None,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(Json(ApiResponse::success(item)))
}

pub async fn update_order_item(
    State(state): State<AppState>,
    Path((order_id, item_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateOrderItemRequest>,
) -> Result<Json<ApiResponse<OrderItem>>, axum::http::StatusCode> {
    let row = sqlx::query(
        r#"
        UPDATE order_items
        SET
            quantity = COALESCE($1, quantity),
            notes = COALESCE($2, notes),
            status = COALESCE($3, status),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $4 AND order_id = $5
        RETURNING id, order_id, menu_item_id, quantity, unit_price, status, notes, void_reason, void_by, created_at, updated_at
        "#
    )
    .bind(request.quantity)
    .bind(&request.notes)
    .bind(request.preparation_status.as_ref().map(|s| if s == "Preparing" { "Fired".to_string() } else { s.clone() }))
    .bind(item_id)
    .bind(order_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update order item: {}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let currency = kipko_core::money::currencies::ksh();
            let item = OrderItem {
                id: row.get("id"),
                order_id: row.get("order_id"),
                menu_item_id: row.get("menu_item_id"),
                quantity: row.get("quantity"),
                unit_price: kipko_core::Money::new(row.get("unit_price"), currency.clone()).unwrap(),
                status: match row.get::<&str, _>("status") {
                    "Pending" => OrderItemStatus::Pending,
                    "Fired" => OrderItemStatus::Fired,
                    "Ready" => OrderItemStatus::Ready,
                    "Delivered" => OrderItemStatus::Delivered,
                    "Voided" => OrderItemStatus::Voided,
                    _ => OrderItemStatus::Pending,
                },
                notes: row.get("notes"),
                void_reason: row.get("void_reason"),
                void_by: row.get("void_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(item)))
        }
        None => Err(axum::http::StatusCode::NOT_FOUND),
    }
}

pub async fn delete_order_item(
    State(state): State<AppState>,
    Path((order_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<()>>, axum::http::StatusCode> {
    let result = sqlx::query(
        "DELETE FROM order_items WHERE id = $1 AND order_id = $2"
    )
    .bind(item_id)
    .bind(order_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete order item: {}", e);
        axum::http::StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() == 0 {
        return Err(axum::http::StatusCode::NOT_FOUND);
    }

    Ok(Json(ApiResponse::success(())))
}
