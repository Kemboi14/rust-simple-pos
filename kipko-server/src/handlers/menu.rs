//! Menu management handlers

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

/// Menu item creation request
#[derive(Debug, Deserialize)]
pub struct CreateMenuItemRequest {
    pub category_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: rust_decimal::Decimal,
    pub tax_rate: rust_decimal::Decimal,
    pub preparation_time_minutes: Option<i32>,
    pub display_order: Option<i32>,
    pub image_url: Option<String>,
    pub stock_quantity: Option<i32>,
    pub low_stock_threshold: Option<i32>,
}

/// Menu item update request
#[derive(Debug, Deserialize)]
pub struct UpdateMenuItemRequest {
    pub category_id: Option<Uuid>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<rust_decimal::Decimal>,
    pub tax_rate: Option<rust_decimal::Decimal>,
    pub is_available: Option<bool>,
    pub preparation_time_minutes: Option<i32>,
    pub display_order: Option<i32>,
    pub image_url: Option<String>,
    pub stock_quantity: Option<i32>,
    pub low_stock_threshold: Option<i32>,
}

/// Menu category creation request
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct CreateCategoryRequest {
    pub name: String,
    pub description: Option<String>,
    pub display_order: Option<i32>,
}

/// Get all menu categories
pub async fn get_menu_categories(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<MenuItemCategory>>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, description, display_order, created_at, updated_at
        FROM menu_item_categories 
        ORDER BY display_order, name
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch menu categories: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let categories: Vec<MenuItemCategory> = rows.into_iter().map(|row| MenuItemCategory {
        id: row.get("id"),
        name: row.get("name"),
        description: row.get("description"),
        display_order: row.get("display_order"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect();

    Ok(Json(ApiResponse::success(categories)))
}

/// Get all menu items
pub async fn get_menu_items(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<MenuItem>>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT
            mi.id, mi.category_id, mi.name, mi.description,
            mi.price, mi.tax_rate, mi.is_available,
            mi.preparation_time_minutes, mi.display_order,
            mi.image_url, mi.stock_quantity, mi.low_stock_threshold,
            mi.created_at, mi.updated_at
        FROM menu_items mi
        ORDER BY mi.display_order, mi.name
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch menu items: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let items: Vec<MenuItem> = rows.into_iter().map(|row| MenuItem {
        id: row.get("id"),
        category_id: row.get("category_id"),
        name: row.get("name"),
        description: row.get("description"),
        price: kipko_core::money::Money::new(row.get("price"), kipko_core::money::currencies::ksh()).unwrap(),
        tax_rate: row.get("tax_rate"),
        is_available: row.get("is_available"),
        preparation_time_minutes: row.get("preparation_time_minutes"),
        display_order: row.get("display_order"),
        image_url: row.get("image_url"),
        stock_quantity: row.get("stock_quantity"),
        low_stock_threshold: row.get("low_stock_threshold"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect();

    Ok(Json(ApiResponse::success(items)))
}

/// Get a single menu item by ID
pub async fn get_menu_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<MenuItem>>, StatusCode> {
    let row = sqlx::query(
        r#"
        SELECT
            id, category_id, name, description,
            price, tax_rate, is_available,
            preparation_time_minutes, display_order,
            image_url, stock_quantity, low_stock_threshold,
            created_at, updated_at
        FROM menu_items
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch menu item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let item = MenuItem {
                id: row.get("id"),
                category_id: row.get("category_id"),
                name: row.get("name"),
                description: row.get("description"),
                price: kipko_core::money::Money::new(row.get("price"), kipko_core::money::currencies::ksh()).unwrap(),
                tax_rate: row.get("tax_rate"),
                is_available: row.get("is_available"),
                preparation_time_minutes: row.get("preparation_time_minutes"),
                display_order: row.get("display_order"),
                image_url: row.get("image_url"),
                stock_quantity: row.get("stock_quantity"),
                low_stock_threshold: row.get("low_stock_threshold"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(item)))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Create a new menu item
pub async fn create_menu_item(
    State(state): State<AppState>,
    Json(request): Json<CreateMenuItemRequest>,
) -> Result<Json<ApiResponse<MenuItem>>, StatusCode> {
    let row = sqlx::query(
        r#"
        INSERT INTO menu_items (category_id, name, description, price, tax_rate, preparation_time_minutes, display_order, image_url, stock_quantity, low_stock_threshold)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING
            id, category_id, name, description,
            price, tax_rate, is_available,
            preparation_time_minutes, display_order,
            image_url, stock_quantity, low_stock_threshold,
            created_at, updated_at
        "#
    )
    .bind(request.category_id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(request.price)
    .bind(request.tax_rate)
    .bind(request.preparation_time_minutes)
    .bind(request.display_order.unwrap_or(0))
    .bind(&request.image_url)
    .bind(request.stock_quantity.unwrap_or(0))
    .bind(request.low_stock_threshold.unwrap_or(10))
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create menu item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let item = MenuItem {
        id: row.get("id"),
        category_id: row.get("category_id"),
        name: row.get("name"),
        description: row.get("description"),
        price: kipko_core::money::Money::new(row.get("price"), kipko_core::money::currencies::ksh()).unwrap(),
        tax_rate: row.get("tax_rate"),
        is_available: row.get("is_available"),
        preparation_time_minutes: row.get("preparation_time_minutes"),
        display_order: row.get("display_order"),
        image_url: row.get("image_url"),
        stock_quantity: row.get("stock_quantity"),
        low_stock_threshold: row.get("low_stock_threshold"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(Json(ApiResponse::success(item)))
}

/// Update a menu item
pub async fn update_menu_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateMenuItemRequest>,
) -> Result<Json<ApiResponse<MenuItem>>, StatusCode> {
    let row = sqlx::query(
        r#"
        UPDATE menu_items
        SET
            category_id = COALESCE($2, category_id),
            name = COALESCE($3, name),
            description = COALESCE($4, description),
            price = COALESCE($5, price),
            tax_rate = COALESCE($6, tax_rate),
            is_available = COALESCE($7, is_available),
            preparation_time_minutes = COALESCE($8, preparation_time_minutes),
            display_order = COALESCE($9, display_order),
            image_url = COALESCE($10, image_url),
            stock_quantity = COALESCE($11, stock_quantity),
            low_stock_threshold = COALESCE($12, low_stock_threshold),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        RETURNING
            id, category_id, name, description,
            price, tax_rate, is_available,
            preparation_time_minutes, display_order,
            image_url, stock_quantity, low_stock_threshold,
            created_at, updated_at
        "#
    )
    .bind(id)
    .bind(request.category_id)
    .bind(&request.name)
    .bind(&request.description)
    .bind(request.price)
    .bind(request.tax_rate)
    .bind(request.is_available)
    .bind(request.preparation_time_minutes)
    .bind(request.display_order)
    .bind(&request.image_url)
    .bind(request.stock_quantity)
    .bind(request.low_stock_threshold)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update menu item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let item = MenuItem {
                id: row.get("id"),
                category_id: row.get("category_id"),
                name: row.get("name"),
                description: row.get("description"),
                price: kipko_core::money::Money::new(row.get("price"), kipko_core::money::currencies::ksh()).unwrap(),
                tax_rate: row.get("tax_rate"),
                is_available: row.get("is_available"),
                preparation_time_minutes: row.get("preparation_time_minutes"),
                display_order: row.get("display_order"),
                image_url: row.get("image_url"),
                stock_quantity: row.get("stock_quantity"),
                low_stock_threshold: row.get("low_stock_threshold"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(item)))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Delete a menu item
pub async fn delete_menu_item(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM menu_items WHERE id = $1"
    )
    .bind(id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete menu item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
