//! API Service
//! 
//! This service handles communication with the kipko-server REST API.

use std::collections::HashMap;
use kipko_core::models::*;

#[derive(Clone)]
pub struct ApiService;

impl ApiService {
    const BASE_URL: &'static str = "http://localhost:8080/api/v1";

    // Tables API
    pub async fn get_tables() -> Result<Vec<Table>, String> {
        let url = format!("{}/tables", Self::BASE_URL);
        
        // For now, return mock data until we implement proper HTTP client
        Ok(vec![
            Table {
                id: uuid::Uuid::new_v4(),
                number: 1,
                capacity: 4,
                location: Some("Main Floor".to_string()),
                status: TableStatus::Empty,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Table {
                id: uuid::Uuid::new_v4(),
                number: 2,
                capacity: 2,
                location: Some("Main Floor".to_string()),
                status: TableStatus::Occupied,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Table {
                id: uuid::Uuid::new_v4(),
                number: 3,
                capacity: 6,
                location: Some("Patio".to_string()),
                status: TableStatus::Dirty,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Table {
                id: uuid::Uuid::new_v4(),
                number: 4,
                capacity: 4,
                location: Some("Main Floor".to_string()),
                status: TableStatus::Reserved,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ])
    }

    pub async fn occupy_table(table_id: uuid::Uuid) -> Result<(), String> {
        // TODO: Implement actual API call
        println!("Occupy table: {}", table_id);
        Ok(())
    }

    pub async fn clear_table(table_id: uuid::Uuid) -> Result<(), String> {
        // TODO: Implement actual API call
        println!("Clear table: {}", table_id);
        Ok(())
    }

    pub async fn clean_table(table_id: uuid::Uuid) -> Result<(), String> {
        // TODO: Implement actual API call
        println!("Clean table: {}", table_id);
        Ok(())
    }

    // Orders API
    pub async fn get_orders() -> Result<Vec<Order>, String> {
        // TODO: Implement actual API call
        Ok(vec![
            Order {
                id: uuid::Uuid::new_v4(),
                table_id: uuid::Uuid::new_v4(),
                staff_id: uuid::Uuid::new_v4(),
                status: OrderStatus::Open,
                subtotal: kipko_core::money::Money::new(rust_decimal::Decimal::new(2500, 2), "USD".to_string()).unwrap(),
                tax_amount: kipko_core::money::Money::new(rust_decimal::Decimal::new(200, 2), "USD".to_string()).unwrap(),
                total_amount: kipko_core::money::Money::new(rust_decimal::Decimal::new(2700, 2), "USD".to_string()).unwrap(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ])
    }

    pub async fn create_order(table_id: Option<uuid::Uuid>, staff_id: Option<uuid::Uuid>) -> Result<Order, String> {
        // TODO: Implement actual API call
        Ok(Order {
            id: uuid::Uuid::new_v4(),
            table_id: table_id.unwrap_or(uuid::Uuid::new_v4()),
            staff_id: staff_id.unwrap_or(uuid::Uuid::new_v4()),
            status: OrderStatus::Open,
            subtotal: kipko_core::money::Money::new(rust_decimal::Decimal::ZERO, "USD".to_string()).unwrap(),
            tax_amount: kipko_core::money::Money::new(rust_decimal::Decimal::ZERO, "USD".to_string()).unwrap(),
            total_amount: kipko_core::money::Money::new(rust_decimal::Decimal::ZERO, "USD".to_string()).unwrap(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    pub async fn get_order_items(order_id: uuid::Uuid) -> Result<Vec<OrderItem>, String> {
        // TODO: Implement actual API call
        Ok(vec![
            OrderItem {
                id: uuid::Uuid::new_v4(),
                order_id,
                menu_item_id: uuid::Uuid::new_v4(),
                quantity: 2,
                unit_price: kipko_core::money::Money::new(rust_decimal::Decimal::new(1250, 2), "USD".to_string()).unwrap(),
                status: kipko_core::models::OrderItemStatus::Delivered,
                notes: Some("Extra cheese".to_string()),
                void_reason: None,
                void_by: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ])
    }

    pub async fn close_order(order_id: uuid::Uuid) -> Result<(), String> {
        // TODO: Implement actual API call
        println!("Close order: {}", order_id);
        Ok(())
    }

    // Menu API
    pub async fn get_menu_categories() -> Result<Vec<MenuItemCategory>, String> {
        // TODO: Implement actual API call
        Ok(vec![
            MenuItemCategory {
                id: uuid::Uuid::new_v4(),
                name: "Appetizers".to_string(),
                description: Some("Start your meal with these delicious options".to_string()),
                display_order: 1,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            MenuItemCategory {
                id: uuid::Uuid::new_v4(),
                name: "Main Courses".to_string(),
                description: Some("Hearty main dishes to satisfy your hunger".to_string()),
                display_order: 2,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            MenuItemCategory {
                id: uuid::Uuid::new_v4(),
                name: "Desserts".to_string(),
                description: Some("Sweet endings to your meal".to_string()),
                display_order: 3,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ])
    }

    pub async fn get_menu_items() -> Result<Vec<MenuItem>, String> {
        // TODO: Implement actual API call
        Ok(vec![
            MenuItem {
                id: uuid::Uuid::new_v4(),
                category_id: uuid::Uuid::new_v4(),
                name: "Caesar Salad".to_string(),
                description: Some("Fresh romaine lettuce with caesar dressing and croutons".to_string()),
                price: kipko_core::money::Money::new(rust_decimal::Decimal::new(899, 2), "USD".to_string()).unwrap(),
                tax_rate: rust_decimal::Decimal::new(8, 2),
                is_available: true,
                preparation_time_minutes: Some(10),
                display_order: 1,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            MenuItem {
                id: uuid::Uuid::new_v4(),
                category_id: uuid::Uuid::new_v4(),
                name: "Grilled Salmon".to_string(),
                description: Some("Atlantic salmon grilled to perfection with lemon butter".to_string()),
                price: kipko_core::money::Money::new(rust_decimal::Decimal::new(2499, 2), "USD".to_string()).unwrap(),
                tax_rate: rust_decimal::Decimal::new(8, 2),
                is_available: true,
                preparation_time_minutes: Some(20),
                display_order: 1,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            MenuItem {
                id: uuid::Uuid::new_v4(),
                category_id: uuid::Uuid::new_v4(),
                name: "Chocolate Cake".to_string(),
                description: Some("Rich chocolate cake with chocolate ganache".to_string()),
                price: kipko_core::money::Money::new(rust_decimal::Decimal::new(699, 2), "USD".to_string()).unwrap(),
                tax_rate: rust_decimal::Decimal::new(8, 2),
                is_available: false,
                preparation_time_minutes: Some(5),
                display_order: 1,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ])
    }

    // Staff API
    pub async fn get_staff() -> Result<Vec<Staff>, String> {
        // TODO: Implement actual API call
        Ok(vec![
            Staff {
                id: uuid::Uuid::new_v4(),
                name: "John Doe".to_string(),
                email: "john@kipko.com".to_string(),
                role: StaffRole::Manager,
                is_active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Staff {
                id: uuid::Uuid::new_v4(),
                name: "Jane Smith".to_string(),
                email: "jane@kipko.com".to_string(),
                role: StaffRole::Server,
                is_active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Staff {
                id: uuid::Uuid::new_v4(),
                name: "Mike Johnson".to_string(),
                email: "mike@kipko.com".to_string(),
                role: StaffRole::Kitchen,
                is_active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ])
    }
}
