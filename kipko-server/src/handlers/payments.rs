//! Payment management handlers

use crate::{AppState, ApiResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use kipko_core::{Payment, PaymentMethod, PaymentStatus};

/// Payment creation request
#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub order_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub method: String,
    pub transaction_id: Option<String>,
}

/// Get all payments
pub async fn get_payments(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Payment>>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT
            id, order_id, amount, method, status, transaction_id, created_at, updated_at
        FROM payments
        ORDER BY created_at DESC
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
        amount: row.get("amount"),
        method: match row.get::<&str, _>("method") {
            "Cash" => PaymentMethod::Cash,
            "Card" => PaymentMethod::Card,
            "MobileMoney" => PaymentMethod::MobileMoney,
            "Mpesa" => PaymentMethod::Mpesa,
            _ => PaymentMethod::Cash,
        },
        status: match row.get::<&str, _>("status") {
            "Pending" => PaymentStatus::Pending,
            "Completed" => PaymentStatus::Completed,
            "Failed" => PaymentStatus::Failed,
            "Refunded" => PaymentStatus::Refunded,
            _ => PaymentStatus::Pending,
        },
        transaction_id: row.get("transaction_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
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
            id, order_id, amount, method, status, transaction_id, created_at, updated_at
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
                amount: row.get("amount"),
                method: match row.get::<&str, _>("method") {
                    "Cash" => PaymentMethod::Cash,
                    "Card" => PaymentMethod::Card,
                    "MobileMoney" => PaymentMethod::MobileMoney,
                    "Mpesa" => PaymentMethod::Mpesa,
                    _ => PaymentMethod::Cash,
                },
                status: match row.get::<&str, _>("status") {
                    "Pending" => PaymentStatus::Pending,
                    "Completed" => PaymentStatus::Completed,
                    "Failed" => PaymentStatus::Failed,
                    "Refunded" => PaymentStatus::Refunded,
                    _ => PaymentStatus::Pending,
                },
                transaction_id: row.get("transaction_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(payment)))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Get payments for a specific order
pub async fn get_order_payments(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<Payment>>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT 
            id, order_id, amount, method, status, transaction_id, created_at, updated_at
        FROM payments 
        WHERE order_id = $1
        ORDER BY created_at DESC
        "#
    )
    .bind(order_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch order payments: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let payments: Vec<Payment> = rows.into_iter().map(|row| Payment {
        id: row.get("id"),
        order_id: row.get("order_id"),
        amount: row.get("amount"),
        method: match row.get::<&str, _>("method") {
            "Cash" => PaymentMethod::Cash,
            "Card" => PaymentMethod::Card,
            "MobileMoney" => PaymentMethod::MobileMoney,
            "Mpesa" => PaymentMethod::Mpesa,
            _ => PaymentMethod::Cash,
        },
        status: match row.get::<&str, _>("status") {
            "Pending" => PaymentStatus::Pending,
            "Completed" => PaymentStatus::Completed,
            "Failed" => PaymentStatus::Failed,
            "Refunded" => PaymentStatus::Refunded,
            _ => PaymentStatus::Pending,
        },
        transaction_id: row.get("transaction_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect();

    Ok(Json(ApiResponse::success(payments)))
}

/// Create a new payment
pub async fn create_payment(
    State(state): State<AppState>,
    Json(request): Json<CreatePaymentRequest>,
) -> Result<Json<ApiResponse<Payment>>, StatusCode> {
    let row = sqlx::query(
        r#"
        INSERT INTO payments (order_id, amount, method, status, transaction_id)
        VALUES ($1, $2, $3, 'Pending', $4)
        RETURNING 
            id, order_id, amount, method, status, transaction_id, created_at, updated_at
        "#
    )
    .bind(request.order_id)
    .bind(request.amount)
    .bind(&request.method)
    .bind(&request.transaction_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create payment: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let payment = Payment {
        id: row.get("id"),
        order_id: row.get("order_id"),
        amount: row.get("amount"),
        method: match row.get::<&str, _>("method") {
            "Cash" => PaymentMethod::Cash,
            "Card" => PaymentMethod::Card,
            "MobileMoney" => PaymentMethod::MobileMoney,
            "Mpesa" => PaymentMethod::Mpesa,
            _ => PaymentMethod::Cash,
        },
        status: PaymentStatus::Pending,
        transaction_id: row.get("transaction_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(Json(ApiResponse::success(payment)))
}

/// Complete a payment
pub async fn complete_payment(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<serde_json::Value>,
) -> Result<Json<ApiResponse<Payment>>, StatusCode> {
    let transaction_id = request.get("transaction_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let row = sqlx::query(
        r#"
        UPDATE payments
        SET status = 'Completed', transaction_id = $2, updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        RETURNING 
            id, order_id, amount, method, status, transaction_id, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(transaction_id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to complete payment: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let payment = Payment {
        id: row.get("id"),
        order_id: row.get("order_id"),
        amount: row.get("amount"),
        method: match row.get::<&str, _>("method") {
            "Cash" => PaymentMethod::Cash,
            "Card" => PaymentMethod::Card,
            "MobileMoney" => PaymentMethod::MobileMoney,
            "Mpesa" => PaymentMethod::Mpesa,
            _ => PaymentMethod::Cash,
        },
        status: PaymentStatus::Completed,
        transaction_id: row.get("transaction_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(Json(ApiResponse::success(payment)))
}
