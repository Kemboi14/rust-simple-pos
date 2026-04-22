//! Server-specific models and DTOs
//! 
//! This module contains models specific to the server layer that may differ
//! from the core domain models for API serialization/deserialization.

use serde::Serialize;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// Re-export core models for convenience
pub use kipko_core::models::*;

/// API request/response models for the server layer

/// Table list response
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct TableListResponse {
    pub tables: Vec<Table>,
    pub total: usize,
}

/// Order summary response
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct OrderSummaryResponse {
    pub id: Uuid,
    pub table_number: i32,
    pub staff_name: String,
    pub status: String,
    pub total_amount: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
}

/// Order detail response with items
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct OrderDetailResponse {
    pub order: Order,
    pub items: Vec<OrderItem>,
    pub table: Table,
    pub staff: Staff,
}

/// Menu item with category
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct MenuItemWithCategory {
    pub id: Uuid,
    pub category_name: String,
    pub name: String,
    pub description: Option<String>,
    pub price: rust_decimal::Decimal,
    pub tax_rate: rust_decimal::Decimal,
    pub is_available: bool,
    pub preparation_time_minutes: Option<i32>,
    pub display_order: i32,
}

/// Payment summary
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct PaymentSummary {
    pub payment: Payment,
    pub order_total: rust_decimal::Decimal,
    pub table_number: i32,
}

/// Dashboard statistics
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct DashboardStats {
    pub total_orders_today: i64,
    pub total_revenue_today: rust_decimal::Decimal,
    pub active_tables: i64,
    pub pending_orders: i64,
}

/// Financial summary
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct FinancialSummary {
    pub period: String,
    pub gross_revenue: rust_decimal::Decimal,
    pub tax_collected: rust_decimal::Decimal,
    pub net_revenue: rust_decimal::Decimal,
    pub cash_sales: rust_decimal::Decimal,
    pub card_sales: rust_decimal::Decimal,
    pub tips_collected: rust_decimal::Decimal,
}
