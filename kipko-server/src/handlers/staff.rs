//! Staff management handlers

use crate::{AppState, ApiResponse};
use actix_web::{web, HttpResponse, Result};
use tracing::error;
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
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
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
        error!("Failed to fetch staff: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch staff"}))
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

    Ok(HttpResponse::Ok().json(ApiResponse::success(staff)))
}

/// Get a single staff member by ID
pub async fn get_staff_member(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let id = path.into_inner();

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
        error!("Failed to fetch staff member: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch staff member"}))
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
            Ok(HttpResponse::Ok().json(ApiResponse::success(staff)))
        },
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Staff member not found"}))),
    }
}

/// Create a new staff member
pub async fn create_staff(
    state: web::Data<AppState>,
    request: web::Json<CreateStaffRequest>,
) -> Result<HttpResponse> {
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
        error!("Failed to create staff member: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to create staff member"}))
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

    Ok(HttpResponse::Ok().json(ApiResponse::success(staff)))
}

/// Update a staff member
pub async fn update_staff(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateStaffRequest>,
) -> Result<HttpResponse> {
    let id = path.into_inner();

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
        error!("Failed to update staff member: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to update staff member"}))
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
            Ok(HttpResponse::Ok().json(ApiResponse::success(staff)))
        },
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Staff member not found"}))),
    }
}

/// Delete a staff member
pub async fn delete_staff(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let id = path.into_inner();

    let result = sqlx::query(
        "DELETE FROM staff WHERE id = $1"
    )
    .bind(id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to delete staff member: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to delete staff member"}))
    })?;

    if result.rows_affected() > 0 {
        Ok(HttpResponse::NoContent().finish())
    } else {
        Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Staff member not found"})))
    }
}
