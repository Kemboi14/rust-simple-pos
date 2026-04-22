//! Order management handlers

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

/// Order creation request
#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub table_id: Uuid,
    pub staff_id: Uuid,
    pub order_type: Option<String>,
    pub customer_id: Option<Uuid>,
    pub delivery_address: Option<String>,
}

/// Order update request
#[derive(Debug, Deserialize)]
pub struct UpdateOrderRequest {
    pub status: Option<String>,
}

/// Order item creation request
#[derive(Debug, Deserialize)]
pub struct AddOrderItemRequest {
    pub menu_item_id: Uuid,
    pub quantity: i32,
    pub notes: Option<String>,
}

/// Order item update request
#[derive(Debug, Deserialize)]
pub struct UpdateOrderItemRequest {
    pub quantity: Option<i32>,
    pub status: Option<String>,
    pub notes: Option<String>,
}

/// Tax calculation request
#[derive(Debug, Deserialize)]
pub struct CalculateTaxRequest {
    #[allow(dead_code)]
    pub exemption_id: Option<Uuid>,
}

/// Get all orders
pub async fn get_orders(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Order>>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT 
            id, table_id, staff_id, status::text as status, order_type::text as order_type,
            subtotal, tax_amount, total_amount,
            delivery_address, delivery_fee, customer_id, location_id,
            created_at, updated_at
        FROM orders
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch orders: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let currency = kipko_core::money::currencies::ksh();
    let orders: Vec<Order> = rows.into_iter().map(|row| Order {
        id: row.get("id"),
        table_id: row.get("table_id"),
        staff_id: row.get("staff_id"),
        status: match row.get::<&str, _>("status") {
            "Open" => OrderStatus::Open,
            "Closed" => OrderStatus::Closed,
            "Cancelled" => OrderStatus::Cancelled,
            _ => OrderStatus::Open,
        },
        order_type: match row.get::<Option<&str>, _>("order_type") {
            Some("Takeout") => kipko_core::OrderType::Takeout,
            Some("Delivery") => kipko_core::OrderType::Delivery,
            _ => kipko_core::OrderType::DineIn,
        },
        subtotal: kipko_core::Money::new(row.get("subtotal"), currency.clone()).unwrap(),
        tax_amount: kipko_core::Money::new(row.get("tax_amount"), currency.clone()).unwrap(),
        total_amount: kipko_core::Money::new(row.get("total_amount"), currency.clone()).unwrap(),
        delivery_address: row.get("delivery_address"),
        delivery_fee: kipko_core::Money::new(row.get::<Option<rust_decimal::Decimal>, _>("delivery_fee").unwrap_or(rust_decimal::Decimal::ZERO), currency.clone()).unwrap(),
        customer_id: row.get("customer_id"),
        location_id: row.get("location_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect();

    Ok(Json(ApiResponse::success(orders)))
}

/// Get a single order by ID
pub async fn get_order(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Order>>, StatusCode> {
    let row = sqlx::query(
        r#"
        SELECT 
            id, table_id, staff_id, status::text as status, order_type::text as order_type,
            subtotal, tax_amount, total_amount,
            delivery_address, delivery_fee, customer_id, location_id,
            created_at, updated_at
        FROM orders
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch order: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let currency = kipko_core::money::currencies::ksh();
            let order = Order {
                id: row.get("id"),
                table_id: row.get("table_id"),
                staff_id: row.get("staff_id"),
                status: match row.get::<&str, _>("status") {
                    "Open" => OrderStatus::Open,
                    "Closed" => OrderStatus::Closed,
                    "Cancelled" => OrderStatus::Cancelled,
                    _ => OrderStatus::Open,
                },
                order_type: match row.get::<Option<&str>, _>("order_type") {
                    Some("Takeout") => kipko_core::OrderType::Takeout,
                    Some("Delivery") => kipko_core::OrderType::Delivery,
                    _ => kipko_core::OrderType::DineIn,
                },
                subtotal: kipko_core::Money::new(row.get("subtotal"), currency.clone()).unwrap(),
                tax_amount: kipko_core::Money::new(row.get("tax_amount"), currency.clone()).unwrap(),
                total_amount: kipko_core::Money::new(row.get("total_amount"), currency.clone()).unwrap(),
                delivery_address: row.get("delivery_address"),
                delivery_fee: kipko_core::Money::new(row.get::<Option<rust_decimal::Decimal>, _>("delivery_fee").unwrap_or(rust_decimal::Decimal::ZERO), currency.clone()).unwrap(),
                customer_id: row.get("customer_id"),
                location_id: row.get("location_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(order)))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Create a new order
pub async fn create_order(
    State(state): State<AppState>,
    Json(request): Json<CreateOrderRequest>,
) -> Result<Json<ApiResponse<Order>>, StatusCode> {
    let order_type = request.order_type.unwrap_or_else(|| "DineIn".to_string());
    let delivery_fee = if order_type == "Delivery" { rust_decimal::Decimal::from(100) } else { rust_decimal::Decimal::ZERO };

    let row = sqlx::query(
        r#"
        INSERT INTO orders (table_id, staff_id, order_type, customer_id, delivery_address, delivery_fee)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING
            id, table_id, staff_id, status::text as status, order_type::text as order_type,
            subtotal, tax_amount, total_amount,
            delivery_address, delivery_fee, customer_id, location_id,
            created_at, updated_at
        "#
    )
    .bind(request.table_id)
    .bind(request.staff_id)
    .bind(&order_type)
    .bind(request.customer_id)
    .bind(&request.delivery_address)
    .bind(delivery_fee)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to create order: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let currency = kipko_core::money::currencies::ksh();
    let order = Order {
        id: row.get("id"),
        table_id: row.get("table_id"),
        staff_id: row.get("staff_id"),
        status: match row.get::<&str, _>("status") {
            "Open" => OrderStatus::Open,
            "Closed" => OrderStatus::Closed,
            "Cancelled" => OrderStatus::Cancelled,
            _ => OrderStatus::Open,
        },
        order_type: match row.get::<&str, _>("order_type") {
            "Takeout" => kipko_core::OrderType::Takeout,
            "Delivery" => kipko_core::OrderType::Delivery,
            _ => kipko_core::OrderType::DineIn,
        },
        subtotal: kipko_core::Money::new(row.get("subtotal"), currency.clone()).unwrap(),
        tax_amount: kipko_core::Money::new(row.get("tax_amount"), currency.clone()).unwrap(),
        total_amount: kipko_core::Money::new(row.get("total_amount"), currency.clone()).unwrap(),
        delivery_address: row.get("delivery_address"),
        delivery_fee: kipko_core::Money::new(row.get::<Option<rust_decimal::Decimal>, _>("delivery_fee").unwrap_or(rust_decimal::Decimal::ZERO), currency.clone()).unwrap(),
        customer_id: row.get("customer_id"),
        location_id: row.get("location_id"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(Json(ApiResponse::success(order)))
}

/// Update an order
pub async fn update_order(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateOrderRequest>,
) -> Result<Json<ApiResponse<Order>>, StatusCode> {
    let row = sqlx::query(
        r#"
        UPDATE orders
        SET
            status = COALESCE($2, status),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        RETURNING
            id, table_id, staff_id, status::text as status, order_type::text as order_type,
            subtotal, tax_amount, total_amount,
            delivery_address, delivery_fee, customer_id, location_id,
            created_at, updated_at
        "#
    )
    .bind(id)
    .bind(&request.status)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update order: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let currency = kipko_core::money::currencies::ksh();
            let order = Order {
                id: row.get("id"),
                table_id: row.get("table_id"),
                staff_id: row.get("staff_id"),
                status: match row.get::<&str, _>("status") {
                    "Open" => OrderStatus::Open,
                    "Closed" => OrderStatus::Closed,
                    "Cancelled" => OrderStatus::Cancelled,
                    _ => OrderStatus::Open,
                },
                order_type: match row.get::<Option<&str>, _>("order_type") {
                    Some("Takeout") => kipko_core::OrderType::Takeout,
                    Some("Delivery") => kipko_core::OrderType::Delivery,
                    _ => kipko_core::OrderType::DineIn,
                },
                subtotal: kipko_core::Money::new(row.get("subtotal"), currency.clone()).unwrap(),
                tax_amount: kipko_core::Money::new(row.get("tax_amount"), currency.clone()).unwrap(),
                total_amount: kipko_core::Money::new(row.get("total_amount"), currency.clone()).unwrap(),
                delivery_address: row.get("delivery_address"),
                delivery_fee: kipko_core::Money::new(row.get::<Option<rust_decimal::Decimal>, _>("delivery_fee").unwrap_or(rust_decimal::Decimal::ZERO), currency.clone()).unwrap(),
                customer_id: row.get("customer_id"),
                location_id: row.get("location_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(order)))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Delete an order
pub async fn delete_order(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM orders WHERE id = $1"
    )
    .bind(id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to delete order: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Get order items
pub async fn get_order_items(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<OrderItem>>>, StatusCode> {
    let rows = sqlx::query(
        r#"
        SELECT 
            id, order_id, menu_item_id, quantity, unit_price, 
            status::text, notes, void_reason, void_by,
            created_at, updated_at
        FROM order_items 
        WHERE order_id = $1
        ORDER BY created_at
        "#
    )
    .bind(order_id)
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch order items: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let items: Vec<OrderItem> = rows.into_iter().map(|row| OrderItem {
        id: row.get("id"),
        order_id: row.get("order_id"),
        menu_item_id: row.get("menu_item_id"),
        quantity: row.get("quantity"),
        unit_price: kipko_core::money::Money::new(row.get("unit_price"), "USD".to_string()).unwrap(),
        status: match row.get::<&str, _>("status") {
            "Pending" => OrderItemStatus::Pending,
            "Fired" => OrderItemStatus::Fired,
            "Ready" => OrderItemStatus::Ready,
            "Delivered" => OrderItemStatus::Delivered,
            "Voided" => OrderItemStatus::Voided,
            _ => OrderItemStatus::Pending,
        },
        notes: row.get("notes"),
        void_reason: row.get("void_reason"),
        void_by: row.get("void_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect();

    Ok(Json(ApiResponse::success(items)))
}

/// Add item to order
pub async fn add_order_item(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
    Json(request): Json<AddOrderItemRequest>,
) -> Result<Json<ApiResponse<OrderItem>>, StatusCode> {
    // Get menu item price
    let menu_item_row = sqlx::query(
        "SELECT price FROM menu_items WHERE id = $1"
    )
    .bind(request.menu_item_id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch menu item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let menu_item_row = menu_item_row.ok_or(StatusCode::BAD_REQUEST)?;

    let row = sqlx::query(
        r#"
        INSERT INTO order_items (order_id, menu_item_id, quantity, unit_price, notes)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING 
            id, order_id, menu_item_id, quantity, unit_price, 
            status, notes, void_reason, void_by,
            created_at, updated_at
        "#
    )
    .bind(order_id)
    .bind(request.menu_item_id)
    .bind(request.quantity)
    .bind(menu_item_row.get::<rust_decimal::Decimal, _>("price"))
    .bind(&request.notes)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to add order item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    let item = OrderItem {
        id: row.get("id"),
        order_id: row.get("order_id"),
        menu_item_id: row.get("menu_item_id"),
        quantity: row.get("quantity"),
        unit_price: kipko_core::money::Money::new(row.get("unit_price"), "USD".to_string()).unwrap(),
        status: match row.get::<&str, _>("status") {
            "Pending" => OrderItemStatus::Pending,
            "Fired" => OrderItemStatus::Fired,
            "Ready" => OrderItemStatus::Ready,
            "Delivered" => OrderItemStatus::Delivered,
            "Voided" => OrderItemStatus::Voided,
            _ => OrderItemStatus::Pending,
        },
        notes: row.get("notes"),
        void_reason: row.get("void_reason"),
        void_by: row.get("void_by"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(Json(ApiResponse::success(item)))
}

/// Update order item
pub async fn update_order_item(
    State(state): State<AppState>,
    Path((order_id, item_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<UpdateOrderItemRequest>,
) -> Result<Json<ApiResponse<OrderItem>>, StatusCode> {
    let row = sqlx::query(
        r#"
        UPDATE order_items 
        SET 
            quantity = COALESCE($3, quantity),
            status = COALESCE($4, status),
            notes = COALESCE($5, notes),
            updated_at = CURRENT_TIMESTAMP
        WHERE order_id = $1 AND id = $2
        RETURNING 
            id, order_id, menu_item_id, quantity, unit_price, 
            status, notes, void_reason, void_by,
            created_at, updated_at
        "#
    )
    .bind(order_id)
    .bind(item_id)
    .bind(request.quantity)
    .bind(&request.status)
    .bind(&request.notes)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to update order item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let item = OrderItem {
                id: row.get("id"),
                order_id: row.get("order_id"),
                menu_item_id: row.get("menu_item_id"),
                quantity: row.get("quantity"),
                unit_price: kipko_core::money::Money::new(row.get("unit_price"), "USD".to_string()).unwrap(),
                status: match row.get::<&str, _>("status") {
                    "Pending" => OrderItemStatus::Pending,
                    "Fired" => OrderItemStatus::Fired,
                    "Ready" => OrderItemStatus::Ready,
                    "Delivered" => OrderItemStatus::Delivered,
                    "Voided" => OrderItemStatus::Voided,
                    _ => OrderItemStatus::Pending,
                },
                notes: row.get("notes"),
                void_reason: row.get("void_reason"),
                void_by: row.get("void_by"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(item)))
        },
        None => Err(StatusCode::NOT_FOUND),
    }
}

/// Remove order item
pub async fn remove_order_item(
    State(state): State<AppState>,
    Path((order_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query(
        "DELETE FROM order_items WHERE order_id = $1 AND id = $2"
    )
    .bind(order_id)
    .bind(item_id)
    .execute(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to remove order item: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    if result.rows_affected() > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Calculate order tax
pub async fn calculate_order_tax(
    State(_state): State<AppState>,
    Path(order_id): Path<Uuid>,
    Json(_request): Json<CalculateTaxRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, StatusCode> {
    // This would integrate with the kipko-core tax calculation engine
    // For now, return a placeholder response
    let response = serde_json::json!({
        "order_id": order_id,
        "subtotal": 0.00,
        "tax_amount": 0.00,
        "total_amount": 0.00,
        "tax_breakdown": []
    });

    Ok(Json(ApiResponse::success(response)))
}

/// Close order
pub async fn close_order(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Order>>, StatusCode> {
    let row = sqlx::query(
        r#"
        UPDATE orders
        SET status = 'Closed', updated_at = CURRENT_TIMESTAMP
        WHERE id = $1 AND status = 'Open'
        RETURNING
            id, table_id, staff_id, status::text as status, order_type::text as order_type,
            subtotal, tax_amount, total_amount,
            delivery_address, delivery_fee, customer_id, location_id,
            created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to close order: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    match row {
        Some(row) => {
            let currency = kipko_core::money::currencies::ksh();
            let order = Order {
                id: row.get("id"),
                table_id: row.get("table_id"),
                staff_id: row.get("staff_id"),
                status: match row.get::<&str, _>("status") {
                    "Open" => OrderStatus::Open,
                    "Closed" => OrderStatus::Closed,
                    "Cancelled" => OrderStatus::Cancelled,
                    _ => OrderStatus::Open,
                },
                order_type: match row.get::<Option<&str>, _>("order_type") {
                    Some("Takeout") => kipko_core::OrderType::Takeout,
                    Some("Delivery") => kipko_core::OrderType::Delivery,
                    _ => kipko_core::OrderType::DineIn,
                },
                subtotal: kipko_core::Money::new(row.get("subtotal"), currency.clone()).unwrap(),
                tax_amount: kipko_core::Money::new(row.get("tax_amount"), currency.clone()).unwrap(),
                total_amount: kipko_core::Money::new(row.get("total_amount"), currency.clone()).unwrap(),
                delivery_address: row.get("delivery_address"),
                delivery_fee: kipko_core::Money::new(row.get::<Option<rust_decimal::Decimal>, _>("delivery_fee").unwrap_or(rust_decimal::Decimal::ZERO), currency.clone()).unwrap(),
                customer_id: row.get("customer_id"),
                location_id: row.get("location_id"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(Json(ApiResponse::success(order)))
        },
        None => Err(StatusCode::BAD_REQUEST),
    }
}
