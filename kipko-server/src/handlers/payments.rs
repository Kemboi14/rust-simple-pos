//! Payment management handlers

use crate::{AppState, ApiResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;
use kipko_core::models::*;

/// Payment creation request
#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub order_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub method: String,
    pub tip_amount: Option<rust_decimal::Decimal>,
    pub staff_id: Uuid,
}

/// Get all payments
pub async fn get_payments(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Payment>>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT 
            id, order_id, amount, method, 
            tip_amount, processed_at, staff_id
        FROM payments 
        ORDER BY processed_at DESC
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch payments: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let payments: Vec<Payment> = rows.into_iter().map(|row| Payment {
        id: row.get("id"),
        order_id: row.get("order_id"),
        amount: kipko_core::money::Money::new(row.get("amount"), "USD".to_string()).unwrap(),
        method: match row.get::<&str, _>("method") {
            "Cash" => PaymentMethod::Cash,
            "Card" => PaymentMethod::Card,
            "Mobile" => PaymentMethod::Mobile,
            "GiftCard" => PaymentMethod::GiftCard,
            _ => PaymentMethod::Cash,
        },
        tip_amount: kipko_core::money::Money::new(row.get("tip_amount"), "USD".to_string()).unwrap(),
        processed_at: row.get("processed_at"),
        staff_id: row.get("staff_id"),
    }).collect();

    Ok(Json(ApiResponse::success(payments)))
}

/// Get a single payment by ID
pub async fn get_payment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Payment>>, StatusCode> {
    let row = sqlx::query(
        r#"
        SELECT 
            id, order_id, amount, method, 
            tip_amount, processed_at, staff_id
        FROM payments 
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch payment: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let payment = Payment {
                id: row.get("id"),
                order_id: row.get("order_id"),
                amount: kipko_core::money::Money::new(row.get("amount"), "USD".to_string()).unwrap(),
                method: match row.get::<&str, _>("method") {
                    "Cash" => PaymentMethod::Cash,
                    "Card" => PaymentMethod::Card,
                    "Mobile" => PaymentMethod::Mobile,
                    "GiftCard" => PaymentMethod::GiftCard,
                    _ => PaymentMethod::Cash,
                },
                tip_amount: kipko_core::money::Money::new(row.get("tip_amount"), "USD".to_string()).unwrap(),
                processed_at: row.get("processed_at"),
                staff_id: row.get("staff_id"),
            };
            Ok(Json(ApiResponse::success(payment)))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Create a new payment
pub async fn create_payment(
    State(state): State<AppState>,
    Json(request): Json<CreatePaymentRequest>,
) -> Result<Json<ApiResponse<Payment>>, StatusCode> {
    let row = sqlx::query(
        r#"
        INSERT INTO payments (order_id, amount, method, tip_amount, staff_id)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING 
            id, order_id, amount, method, 
            tip_amount, processed_at, staff_id
        "#
    )
    .bind(request.order_id)
    .bind(request.amount)
    .bind(&request.method)
    .bind(request.tip_amount.unwrap_or(rust_decimal::Decimal::ZERO))
    .bind(request.staff_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create payment: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let payment = Payment {
        id: row.get("id"),
        order_id: row.get("order_id"),
        amount: kipko_core::money::Money::new(row.get("amount"), "USD".to_string()).unwrap(),
        method: match row.get::<&str, _>("method") {
            "Cash" => PaymentMethod::Cash,
            "Card" => PaymentMethod::Card,
            "Mobile" => PaymentMethod::Mobile,
            "GiftCard" => PaymentMethod::GiftCard,
            _ => PaymentMethod::Cash,
        },
        tip_amount: kipko_core::money::Money::new(row.get("tip_amount"), "USD".to_string()).unwrap(),
        processed_at: row.get("processed_at"),
        staff_id: row.get("staff_id"),
    };

    Ok(Json(ApiResponse::success(payment)))
}
