//! Payment management handlers

use crate::{AppState, ApiResponse};
use crate::services::accounting_service::AccountingService;
use actix_web::{web, HttpResponse, Result};
use tracing::error;
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
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
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
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch payments"}))
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

    Ok(HttpResponse::Ok().json(ApiResponse::success(payments)))
}

/// Get a single payment by ID
pub async fn get_payment(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let id = path.into_inner();

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
        error!("Failed to fetch payment: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch payment"}))
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
            Ok(HttpResponse::Ok().json(ApiResponse::success(payment)))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Payment not found"}))),
    }
}

/// Get payments for a specific order
pub async fn get_order_payments(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let order_id = path.into_inner();

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
        error!("Failed to fetch order payments: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch order payments"}))
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

    Ok(HttpResponse::Ok().json(ApiResponse::success(payments)))
}

/// Create a new payment
pub async fn create_payment(
    state: web::Data<AppState>,
    request: web::Json<CreatePaymentRequest>,
) -> Result<HttpResponse> {
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
        error!("Failed to create payment: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to create payment"}))
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

    Ok(HttpResponse::Ok().json(ApiResponse::success(payment)))
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

    let payment_id: Uuid = row.get("id");
    let order_id: Uuid = row.get("order_id");
    let payment_amount: rust_decimal::Decimal = row.get("amount");
    let payment_method: String = row.get("method");

    // Create accounting entries for the payment
    let accounting_service = AccountingService::new(state.db_pool.clone());
    if let Err(e) = accounting_service.create_payment_accounting_entries(payment_id, payment_amount, &payment_method).await {
        tracing::error!("Failed to create accounting entries for payment: {}", e);
        // Don't fail the payment if accounting fails, but log it
    }

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
