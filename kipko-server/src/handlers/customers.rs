//! Customer management handlers

use crate::{AppState, ApiResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use kipko_core::Customer;

#[derive(Debug, Deserialize)]
pub struct CreateCustomerRequest {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomerRequest {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub loyalty_points: Option<i32>,
}

pub async fn get_customers(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Customer>>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, phone, email, loyalty_points, created_at, updated_at
        FROM customers
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch customers: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let customers: Vec<Customer> = rows.into_iter().map(|row| Customer {
        id: row.get("id"),
        name: row.get("name"),
        phone: row.get("phone"),
        email: row.get("email"),
        loyalty_points: row.get("loyalty_points"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect();

    Ok(Json(ApiResponse::success(customers)))
}

pub async fn get_customer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Customer>>, StatusCode> {
    let row = sqlx::query(
        r#"
        SELECT id, name, phone, email, loyalty_points, created_at, updated_at
        FROM customers
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch customer: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let customer = Customer {
                id: row.get("id"),
                name: row.get("name"),
                phone: row.get("phone"),
                email: row.get("email"),
                loyalty_points: row.get("loyalty_points"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(customer)))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_customer(
    State(state): State<AppState>,
    Json(request): Json<CreateCustomerRequest>,
) -> Result<Json<ApiResponse<Customer>>, StatusCode> {
    let row = sqlx::query(
        r#"
        INSERT INTO customers (name, phone, email, loyalty_points)
        VALUES ($1, $2, $3, 0)
        RETURNING id, name, phone, email, loyalty_points, created_at, updated_at
        "#
    )
    .bind(&request.name)
    .bind(&request.phone)
    .bind(&request.email)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create customer: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let customer = Customer {
        id: row.get("id"),
        name: row.get("name"),
        phone: row.get("phone"),
        email: row.get("email"),
        loyalty_points: row.get("loyalty_points"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(Json(ApiResponse::success(customer)))
}

pub async fn update_customer(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateCustomerRequest>,
) -> Result<Json<ApiResponse<Customer>>, StatusCode> {
    let row = sqlx::query(
        r#"
        UPDATE customers
        SET
            name = COALESCE($2, name),
            phone = COALESCE($3, phone),
            email = COALESCE($4, email),
            loyalty_points = COALESCE($5, loyalty_points),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        RETURNING id, name, phone, email, loyalty_points, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(&request.name)
    .bind(&request.phone)
    .bind(&request.email)
    .bind(request.loyalty_points)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update customer: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let customer = Customer {
                id: row.get("id"),
                name: row.get("name"),
                phone: row.get("phone"),
                email: row.get("email"),
                loyalty_points: row.get("loyalty_points"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(customer)))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}
