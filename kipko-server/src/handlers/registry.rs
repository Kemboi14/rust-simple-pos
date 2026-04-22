//! Registry handlers for tracking system events

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

/// Registry entry creation request
#[derive(Debug, Deserialize)]
pub struct CreateRegistryEntryRequest {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub action: String,
    pub details: Option<String>,
    pub created_by: Uuid,
}

/// Get all registry entries
pub async fn get_registry_entries(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<RegistryEntry>>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT 
            id, entity_type, entity_id, action, details, created_by, created_at
        FROM registry_entries
        ORDER BY created_at DESC
        LIMIT 100
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch registry entries: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let entries: Vec<RegistryEntry> = rows.into_iter().map(|row| RegistryEntry {
        id: row.get("id"),
        entity_type: row.get("entity_type"),
        entity_id: row.get("entity_id"),
        action: row.get("action"),
        details: row.get("details"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
    }).collect();

    Ok(Json(ApiResponse::success(entries)))
}

/// Get registry entries for a specific entity
pub async fn get_registry_entries_for_entity(
    State(state): State<AppState>,
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
) -> Result<Json<ApiResponse<Vec<RegistryEntry>>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT 
            id, entity_type, entity_id, action, details, created_by, created_at
        FROM registry_entries
        WHERE entity_type = $1 AND entity_id = $2
        ORDER BY created_at DESC
        "#
    )
    .bind(&entity_type)
    .bind(entity_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch registry entries: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let entries: Vec<RegistryEntry> = rows.into_iter().map(|row| RegistryEntry {
        id: row.get("id"),
        entity_type: row.get("entity_type"),
        entity_id: row.get("entity_id"),
        action: row.get("action"),
        details: row.get("details"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
    }).collect();

    Ok(Json(ApiResponse::success(entries)))
}

/// Create a new registry entry
pub async fn create_registry_entry(
    State(state): State<AppState>,
    Json(request): Json<CreateRegistryEntryRequest>,
) -> Result<Json<ApiResponse<RegistryEntry>>, StatusCode> {
    let row = sqlx::query(
        r#"
        INSERT INTO registry_entries (entity_type, entity_id, action, details, created_by)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, entity_type, entity_id, action, details, created_by, created_at
        "#
    )
    .bind(&request.entity_type)
    .bind(request.entity_id)
    .bind(&request.action)
    .bind(&request.details)
    .bind(request.created_by)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create registry entry: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let entry = RegistryEntry {
        id: row.get("id"),
        entity_type: row.get("entity_type"),
        entity_id: row.get("entity_id"),
        action: row.get("action"),
        details: row.get("details"),
        created_by: row.get("created_by"),
        created_at: row.get("created_at"),
    };

    Ok(Json(ApiResponse::success(entry)))
}
