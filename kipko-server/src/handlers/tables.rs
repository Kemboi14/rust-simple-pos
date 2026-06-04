//! Table management handlers

use crate::{AppState, ApiResponse};
use actix_web::{web, HttpResponse, Result};
use tracing::error as err;
use serde::Deserialize;
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
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let rows = sqlx::query(
        r#"
        SELECT
            id, number, capacity, status::text, location,
            created_at, updated_at
        FROM tables
        ORDER BY number
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch tables: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch tables"}))
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

    Ok(HttpResponse::Ok().json(ApiResponse::success(tables)))
}

/// Get a single table by ID
pub async fn get_table(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let id = path.into_inner();

    let row = sqlx::query(
        r#"
        SELECT
            id, number, capacity, status::text, location,
            created_at, updated_at
        FROM tables
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch table: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch table"}))
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
            Ok(HttpResponse::Ok().json(ApiResponse::success(table)))
        },
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Table not found"}))),
    }
}

/// Create a new table
pub async fn create_table(
    state: web::Data<AppState>,
    request: web::Json<CreateTableRequest>,
) -> Result<HttpResponse> {
    let row = sqlx::query(
        r#"
        INSERT INTO tables (number, capacity, location)
        VALUES ($1, $2, $3)
        RETURNING
            id, number, capacity, status::text, location,
            created_at, updated_at
        "#
    )
    .bind(request.number)
    .bind(request.capacity)
    .bind(&request.location)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to create table: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to create table"}))
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

    Ok(HttpResponse::Ok().json(ApiResponse::success(table)))
}

/// Update a table
pub async fn update_table(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateTableRequest>,
) -> Result<HttpResponse> {
    let id = path.into_inner();

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
            id, number, capacity, status::text, location,
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
        error!("Failed to update table: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to update table"}))
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
            Ok(HttpResponse::Ok().json(ApiResponse::success(table)))
        },
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Table not found"}))),
    }
}

/// Delete a table
pub async fn delete_table(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let id = path.into_inner();

    let result = sqlx::query(
        "DELETE FROM tables WHERE id = $1"
    )
    .bind(id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to delete table: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to delete table"}))
    })?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Table not found"})))
    }
}

/// Occupy a table
pub async fn occupy_table(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let id = path.into_inner();

    let row = sqlx::query(
        r#"
        UPDATE tables
        SET status = 'Occupied', updated_at = CURRENT_TIMESTAMP
        WHERE id = $1 AND status IN ('Empty', 'Dirty')
        RETURNING
            id, number, capacity, status::text, location,
            created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to occupy table: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to occupy table"}))
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
            Ok(HttpResponse::Ok().json(ApiResponse::success(table)))
        },
        None => Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": "Table cannot be occupied in current status"}))),
    }
}

/// Clear a table
pub async fn clear_table(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let id = path.into_inner();

    let row = sqlx::query(
        r#"
        UPDATE tables
        SET status = 'Dirty', updated_at = CURRENT_TIMESTAMP
        WHERE id = $1 AND status = 'Occupied'
        RETURNING
            id, number, capacity, status::text, location,
            created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to clear table: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to clear table"}))
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
            Ok(HttpResponse::Ok().json(ApiResponse::success(table)))
        },
        None => Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": "Table cannot be cleared in current status"}))),
    }
}

/// Clean a table
pub async fn clean_table(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let id = path.into_inner();

    let row = sqlx::query(
        r#"
        UPDATE tables
        SET status = 'Empty', updated_at = CURRENT_TIMESTAMP
        WHERE id = $1 AND status = 'Dirty'
        RETURNING
            id, number, capacity, status::text, location,
            created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to clean table: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to clean table"}))
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
            Ok(HttpResponse::Ok().json(ApiResponse::success(table)))
        },
        None => Ok(HttpResponse::BadRequest().json(serde_json::json!({"error": "Table cannot be cleaned in current status"}))),
    }
}
