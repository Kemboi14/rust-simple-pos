//! Table management handlers

use crate::{AppState, ApiResponse, PaginatedResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;
use kipko_core::models::*;

/// Table creation request
#[derive(Debug, Deserialize)]
pub struct CreateTableRequest {
    pub number: i32,
    pub capacity: i32,
    pub location: Option<String>,
}

/// Table update request
#[derive(Debug, Deserialize)]
pub struct UpdateTableRequest {
    pub number: Option<i32>,
    pub capacity: Option<i32>,
    pub status: Option<String>,
    pub location: Option<String>,
}

/// Get all tables
pub async fn get_tables(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Table>>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT 
            id, number, capacity, status, location,
            created_at, updated_at
        FROM tables 
        ORDER BY number
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch tables: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let tables: Vec<Table> = rows.into_iter().map(|row| Table {
        id: row.get("id"),
        number: row.get("number"),
        capacity: row.get("capacity"),
        status: match row.get::<&str, _>("status") {
            "Empty" => TableStatus::Empty,
            "Occupied" => TableStatus::Occupied,
            "Dirty" => TableStatus::Dirty,
            "Reserved" => TableStatus::Reserved,
            _ => TableStatus::Empty,
        },
        location: row.get("location"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect();

    Ok(Json(ApiResponse::success(tables)))
}

/// Get a single table by ID
pub async fn get_table(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Table>>, StatusCode> {
    let row = sqlx::query(
        r#"
        SELECT 
            id, number, capacity, status, location,
            created_at, updated_at
        FROM tables 
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch table: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let table = Table {
                id: row.get("id"),
                number: row.get("number"),
                capacity: row.get("capacity"),
                status: match row.get::<&str, _>("status") {
                    "Empty" => TableStatus::Empty,
                    "Occupied" => TableStatus::Occupied,
                    "Dirty" => TableStatus::Dirty,
                    "Reserved" => TableStatus::Reserved,
                    _ => TableStatus::Empty,
                },
                location: row.get("location"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(table)))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Create a new table
pub async fn create_table(
    State(state): State<AppState>,
    Json(request): Json<CreateTableRequest>,
) -> Result<Json<ApiResponse<Table>>, StatusCode> {
    let row = sqlx::query(
        r#"
        INSERT INTO tables (number, capacity, location)
        VALUES ($1, $2, $3)
        RETURNING 
            id, number, capacity, status, location,
            created_at, updated_at
        "#
    )
    .bind(request.number)
    .bind(request.capacity)
    .bind(&request.location)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create table: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let table = Table {
        id: row.get("id"),
        number: row.get("number"),
        capacity: row.get("capacity"),
        status: match row.get::<&str, _>("status") {
            "Empty" => TableStatus::Empty,
            "Occupied" => TableStatus::Occupied,
            "Dirty" => TableStatus::Dirty,
            "Reserved" => TableStatus::Reserved,
            _ => TableStatus::Empty,
        },
        location: row.get("location"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(Json(ApiResponse::success(table)))
}

/// Update a table
pub async fn update_table(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateTableRequest>,
) -> Result<Json<ApiResponse<Table>>, StatusCode> {
    let row = sqlx::query(
        r#"
        UPDATE tables 
        SET 
            number = COALESCE($2, number),
            capacity = COALESCE($3, capacity),
            status = COALESCE($4, status),
            location = COALESCE($5, location),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        RETURNING 
            id, number, capacity, status, location,
            created_at, updated_at
        "#
    )
    .bind(id)
    .bind(request.number)
    .bind(request.capacity)
    .bind(&request.status)
    .bind(&request.location)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update table: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let table = Table {
                id: row.get("id"),
                number: row.get("number"),
                capacity: row.get("capacity"),
                status: match row.get::<&str, _>("status") {
                    "Empty" => TableStatus::Empty,
                    "Occupied" => TableStatus::Occupied,
                    "Dirty" => TableStatus::Dirty,
                    "Reserved" => TableStatus::Reserved,
                    _ => TableStatus::Empty,
                },
                location: row.get("location"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(table)))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Delete a table
pub async fn delete_table(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM tables WHERE id = $1"
    )
    .bind(id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete table: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Occupy a table
pub async fn occupy_table(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Table>>, StatusCode> {
    let row = sqlx::query(
        r#"
        UPDATE tables 
        SET status = 'Occupied', updated_at = CURRENT_TIMESTAMP
        WHERE id = $1 AND status IN ('Empty', 'Dirty')
        RETURNING 
            id, number, capacity, status, location,
            created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to occupy table: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let table = Table {
                id: row.get("id"),
                number: row.get("number"),
                capacity: row.get("capacity"),
                status: match row.get::<&str, _>("status") {
                    "Empty" => TableStatus::Empty,
                    "Occupied" => TableStatus::Occupied,
                    "Dirty" => TableStatus::Dirty,
                    "Reserved" => TableStatus::Reserved,
                    _ => TableStatus::Empty,
                },
                location: row.get("location"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(table)))
        },
        None => Err(StatusCode::BAD_REQUEST),
    }
}

/// Clear a table
pub async fn clear_table(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Table>>, StatusCode> {
    let row = sqlx::query(
        r#"
        UPDATE tables 
        SET status = 'Dirty', updated_at = CURRENT_TIMESTAMP
        WHERE id = $1 AND status = 'Occupied'
        RETURNING 
            id, number, capacity, status, location,
            created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to clear table: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let table = Table {
                id: row.get("id"),
                number: row.get("number"),
                capacity: row.get("capacity"),
                status: match row.get::<&str, _>("status") {
                    "Empty" => TableStatus::Empty,
                    "Occupied" => TableStatus::Occupied,
                    "Dirty" => TableStatus::Dirty,
                    "Reserved" => TableStatus::Reserved,
                    _ => TableStatus::Empty,
                },
                location: row.get("location"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(table)))
        },
        None => Err(StatusCode::BAD_REQUEST),
    }
}

/// Clean a table
pub async fn clean_table(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Table>>, StatusCode> {
    let row = sqlx::query(
        r#"
        UPDATE tables 
        SET status = 'Empty', updated_at = CURRENT_TIMESTAMP
        WHERE id = $1 AND status = 'Dirty'
        RETURNING 
            id, number, capacity, status, location,
            created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to clean table: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let table = Table {
                id: row.get("id"),
                number: row.get("number"),
                capacity: row.get("capacity"),
                status: match row.get::<&str, _>("status") {
                    "Empty" => TableStatus::Empty,
                    "Occupied" => TableStatus::Occupied,
                    "Dirty" => TableStatus::Dirty,
                    "Reserved" => TableStatus::Reserved,
                    _ => TableStatus::Empty,
                },
                location: row.get("location"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(table)))
        },
        None => Err(StatusCode::BAD_REQUEST),
    }
}
