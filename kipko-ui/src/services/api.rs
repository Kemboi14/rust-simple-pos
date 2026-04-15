//! API Service
//! 
//! This service handles communication with the kipko-server REST API.

use kipko_core::{Table, TableStatus, Order, OrderStatus, OrderItem, MenuItem, MenuItemCategory, Staff, StaffRole};

#[derive(Clone)]
pub struct ApiService {
    base_url: String,
}

impl Default for ApiService {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiService {
    pub fn new() -> Self {
        Self {
            base_url: std::env::var("API_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
        }
    }

    // Tables API
    pub async fn get_tables(&self) -> Result<Vec<Table>, String> {
        let _url = format!("{}/tables", self.base_url);
        
        // For now, return mock data until we implement proper HTTP client
        // TODO: Replace with actual HTTP call using reqwest
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
            Table {
                id: uuid::Uuid::new_v4(),
                number: 5,
                capacity: 8,
                location: Some("Patio".to_string()),
                status: TableStatus::Empty,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            Table {
                id: uuid::Uuid::new_v4(),
                number: 6,
                capacity: 4,
                location: Some("Bar".to_string()),
                status: TableStatus::Empty,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ])
    }

    pub async fn occupy_table(&self, table_id: uuid::Uuid) -> Result<(), String> {
        let _url = format!("{}/tables/{}/occupy", self.base_url, table_id);
        // TODO: Implement actual HTTP call
        println!("Occupy table: {} at {}", table_id, _url);
        Ok(())
    }

    pub async fn clear_table(&self, table_id: uuid::Uuid) -> Result<(), String> {
        let _url = format!("{}/tables/{}/clear", self.base_url, table_id);
        // TODO: Implement actual HTTP call
        println!("Clear table: {} at {}", table_id, _url);
        Ok(())
    }

    pub async fn clean_table(&self, table_id: uuid::Uuid) -> Result<(), String> {
        let _url = format!("{}/tables/{}/clean", self.base_url, table_id);
        // TODO: Implement actual HTTP call
        println!("Clean table: {} at {}", table_id, _url);
        Ok(())
    }

    // Orders API
    pub async fn get_orders(&self) -> Result<Vec<Order>, String> {
        let _url = format!("{}/orders", self.base_url);
        // TODO: Implement actual HTTP call
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
            Order {
                id: uuid::Uuid::new_v4(),
                table_id: uuid::Uuid::new_v4(),
                staff_id: uuid::Uuid::new_v4(),
                status: OrderStatus::Closed,
                subtotal: kipko_core::money::Money::new(rust_decimal::Decimal::new(1800, 2), "USD".to_string()).unwrap(),
                tax_amount: kipko_core::money::Money::new(rust_decimal::Decimal::new(144, 2), "USD".to_string()).unwrap(),
                total_amount: kipko_core::money::Money::new(rust_decimal::Decimal::new(1944, 2), "USD".to_string()).unwrap(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ])
    }

    pub async fn create_order(&self, table_id: uuid::Uuid, staff_id: uuid::Uuid) -> Result<Order, String> {
        let _url = format!("{}/orders", self.base_url);
        // TODO: Implement actual HTTP call
        Ok(Order {
            id: uuid::Uuid::new_v4(),
            table_id,
            staff_id,
            status: OrderStatus::Open,
            subtotal: kipko_core::money::Money::new(rust_decimal::Decimal::ZERO, "USD".to_string()).unwrap(),
            tax_amount: kipko_core::money::Money::new(rust_decimal::Decimal::ZERO, "USD".to_string()).unwrap(),
            total_amount: kipko_core::money::Money::new(rust_decimal::Decimal::ZERO, "USD".to_string()).unwrap(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
    }

    pub async fn get_order_items(&self, order_id: uuid::Uuid) -> Result<Vec<OrderItem>, String> {
        let _url = format!("{}/orders/{}/items", self.base_url, order_id);
        // TODO: Implement actual HTTP call
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

    pub async fn close_order(&self, order_id: uuid::Uuid) -> Result<(), String> {
        let _url = format!("{}/orders/{}/close", self.base_url, order_id);
        // TODO: Implement actual HTTP call
        println!("Close order: {} at {}", order_id, _url);
        Ok(())
    }

    // Menu API
    pub async fn get_menu_categories(&self) -> Result<Vec<MenuItemCategory>, String> {
        let _url = format!("{}/menu/categories", self.base_url);
        // TODO: Implement actual HTTP call
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
                name: "Beverages".to_string(),
                description: Some("Refreshing drinks and beverages".to_string()),
                display_order: 3,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            MenuItemCategory {
                id: uuid::Uuid::new_v4(),
                name: "Desserts".to_string(),
                description: Some("Sweet endings to your meal".to_string()),
                display_order: 4,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ])
    }

    pub async fn get_menu_items(&self) -> Result<Vec<MenuItem>, String> {
        let _url = format!("{}/menu/items", self.base_url);
        // TODO: Implement actual HTTP call
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
            MenuItem {
                id: uuid::Uuid::new_v4(),
                category_id: uuid::Uuid::new_v4(),
                name: "House Burger".to_string(),
                description: Some("Angus beef patty with lettuce, tomato, and special sauce".to_string()),
                price: kipko_core::money::Money::new(rust_decimal::Decimal::new(1499, 2), "USD".to_string()).unwrap(),
                tax_rate: rust_decimal::Decimal::new(8, 2),
                is_available: true,
                preparation_time_minutes: Some(15),
                display_order: 2,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ])
    }

    // Staff API
    pub async fn get_staff(&self) -> Result<Vec<Staff>, String> {
        let _url = format!("{}/staff", self.base_url);
        // TODO: Implement actual HTTP call
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
            Staff {
                id: uuid::Uuid::new_v4(),
                name: "Sarah Williams".to_string(),
                email: "sarah@kipko.com".to_string(),
                role: StaffRole::Host,
                is_active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        ])
    }
}
