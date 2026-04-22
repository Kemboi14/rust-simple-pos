//! Staff management handlers

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

/// Staff creation request
#[derive(Debug, Deserialize)]
pub struct CreateStaffRequest {
    pub name: String,
    pub email: String,
    pub role: String,
}

/// Staff update request
#[derive(Debug, Deserialize)]
pub struct UpdateStaffRequest {
    pub name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub is_active: Option<bool>,
}

/// Get all staff members
pub async fn get_staff(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Staff>>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT 
            id, name, email, role::text, is_active,
            created_at, updated_at
        FROM staff 
        ORDER BY name
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch staff: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let staff: Vec<Staff> = rows.into_iter().map(|row| Staff {
        id: row.get("id"),
        name: row.get("name"),
        email: row.get("email"),
        role: match row.get::<&str, _>("role") {
            "Server" => StaffRole::Server,
            "Manager" => StaffRole::Manager,
            "Kitchen" => StaffRole::Kitchen,
            "Host" => StaffRole::Host,
            "Admin" => StaffRole::Admin,
            _ => StaffRole::Server,
        },
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect();

    Ok(Json(ApiResponse::success(staff)))
}

/// Get a single staff member by ID
pub async fn get_staff_member(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Staff>>, StatusCode> {
    let row = sqlx::query(
        r#"
        SELECT 
            id, name, email, role::text, is_active,
            created_at, updated_at
        FROM staff 
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch staff member: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let staff = Staff {
                id: row.get("id"),
                name: row.get("name"),
                email: row.get("email"),
                role: match row.get::<&str, _>("role") {
                    "Server" => StaffRole::Server,
                    "Manager" => StaffRole::Manager,
                    "Kitchen" => StaffRole::Kitchen,
                    "Host" => StaffRole::Host,
                    "Admin" => StaffRole::Admin,
                    _ => StaffRole::Server,
                },
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(staff)))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Create a new staff member
pub async fn create_staff(
    State(state): State<AppState>,
    Json(request): Json<CreateStaffRequest>,
) -> Result<Json<ApiResponse<Staff>>, StatusCode> {
    let row = sqlx::query(
        r#"
        INSERT INTO staff (name, email, role)
        VALUES ($1, $2, $3)
        RETURNING 
            id, name, email, role::text, is_active,
            created_at, updated_at
        "#
    )
    .bind(&request.name)
    .bind(&request.email)
    .bind(&request.role)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create staff member: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let staff = Staff {
        id: row.get("id"),
        name: row.get("name"),
        email: row.get("email"),
        role: match row.get::<&str, _>("role") {
            "Server" => StaffRole::Server,
            "Manager" => StaffRole::Manager,
            "Kitchen" => StaffRole::Kitchen,
            "Host" => StaffRole::Host,
            "Admin" => StaffRole::Admin,
            _ => StaffRole::Server,
        },
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(Json(ApiResponse::success(staff)))
}

/// Update a staff member
pub async fn update_staff(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateStaffRequest>,
) -> Result<Json<ApiResponse<Staff>>, StatusCode> {
    let row = sqlx::query(
        r#"
        UPDATE staff 
        SET 
            name = COALESCE($2, name),
            email = COALESCE($3, email),
            role = COALESCE($4, role),
            is_active = COALESCE($5, is_active),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        RETURNING 
            id, name, email, role::text, is_active,
            created_at, updated_at
        "#
    )
    .bind(id)
    .bind(&request.name)
    .bind(&request.email)
    .bind(&request.role)
    .bind(request.is_active)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update staff member: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let staff = Staff {
                id: row.get("id"),
                name: row.get("name"),
                email: row.get("email"),
                role: match row.get::<&str, _>("role") {
                    "Server" => StaffRole::Server,
                    "Manager" => StaffRole::Manager,
                    "Kitchen" => StaffRole::Kitchen,
                    "Host" => StaffRole::Host,
                    "Admin" => StaffRole::Admin,
                    _ => StaffRole::Server,
                },
                is_active: row.get("is_active"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(staff)))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Delete a staff member
pub async fn delete_staff(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM staff WHERE id = $1"
    )
    .bind(id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete staff member: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
