//! Domain models for Kipko POS
//! 
//! This module contains the core domain entities for the restaurant POS system,
//! including tables, menu items, orders, and related business objects.

use crate::money::Money;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Table status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum TableStatus {
    Empty,
    Occupied,
    Dirty,
    Reserved,
}

impl Default for TableStatus {
    fn default() -> Self {
        TableStatus::Empty
    }
}

/// Restaurant table
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Table {
    pub id: Uuid,
    pub number: i32,
    pub capacity: i32,
    pub status: TableStatus,
    pub location: Option<String>, // e.g., "Patio", "Bar", "Main Floor"
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Table {
    pub fn new(number: i32, capacity: i32, location: Option<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            number,
            capacity,
            status: TableStatus::Empty,
            location,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn occupy(&mut self) -> Result<(), String> {
        match self.status {
            TableStatus::Empty | TableStatus::Dirty => {
                self.status = TableStatus::Occupied;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!("Cannot occupy table in status: {:?}", self.status)),
        }
    }

    pub fn clear(&mut self) -> Result<(), String> {
        match self.status {
            TableStatus::Occupied => {
                self.status = TableStatus::Dirty;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!("Cannot clear table in status: {:?}", self.status)),
        }
    }

    pub fn clean(&mut self) -> Result<(), String> {
        match self.status {
            TableStatus::Dirty => {
                self.status = TableStatus::Empty;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!("Cannot clean table in status: {:?}", self.status)),
        }
    }
}

/// Order item status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum OrderItemStatus {
    Pending,
    Fired,    // Sent to kitchen
    Ready,    // Ready for pickup
    Delivered,
    Voided,
}

/// Order item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub menu_item_id: Uuid,
    pub quantity: i32,
    pub unit_price: Money,
    pub status: OrderItemStatus,
    pub notes: Option<String>,
    pub void_reason: Option<String>,
    pub void_by: Option<Uuid>, // Staff member who voided the item
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl OrderItem {
    pub fn new(order_id: Uuid, menu_item_id: Uuid, quantity: i32, unit_price: Money) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            order_id,
            menu_item_id,
            quantity,
            unit_price,
            status: OrderItemStatus::Pending,
            notes: None,
            void_reason: None,
            void_by: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn subtotal(&self) -> Money {
        self.unit_price.multiply(self.quantity.into()).unwrap()
    }

    pub fn fire(&mut self) -> Result<(), String> {
        match self.status {
            OrderItemStatus::Pending => {
                self.status = OrderItemStatus::Fired;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!("Cannot fire item in status: {:?}", self.status)),
        }
    }

    pub fn void(&mut self, reason: String, voided_by: Uuid) -> Result<(), String> {
        match self.status {
            OrderItemStatus::Pending | OrderItemStatus::Fired => {
                self.status = OrderItemStatus::Voided;
                self.void_reason = Some(reason);
                self.void_by = Some(voided_by);
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!("Cannot void item in status: {:?}", self.status)),
        }
    }
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum OrderStatus {
    Open,
    Closed,
    Cancelled,
}

/// Order
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Order {
    pub id: Uuid,
    pub table_id: Uuid,
    pub staff_id: Uuid,
    pub status: OrderStatus,
    pub order_type: OrderType,
    pub subtotal: Money,
    pub tax_amount: Money,
    pub total_amount: Money,
    pub delivery_address: Option<String>,
    pub delivery_fee: Money,
    pub customer_id: Option<Uuid>,
    pub location_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Order {
    pub fn new(table_id: Uuid, staff_id: Uuid) -> Self {
        let now = Utc::now();
        let currency = crate::money::currencies::ksh();
        Self {
            id: Uuid::new_v4(),
            table_id,
            staff_id,
            status: OrderStatus::Open,
            order_type: OrderType::DineIn,
            subtotal: Money::zero(currency.clone()),
            tax_amount: Money::zero(currency.clone()),
            total_amount: Money::zero(currency.clone()),
            delivery_address: None,
            delivery_fee: Money::zero(currency.clone()),
            customer_id: None,
            location_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn calculate_totals(&mut self, items: &[OrderItem]) {
        let currency = crate::money::currencies::ksh();
        self.subtotal = items
            .iter()
            .filter(|item| item.status != OrderItemStatus::Voided)
            .map(|item| item.subtotal())
            .fold(Money::zero(currency.clone()), |acc, subtotal| acc.add(&subtotal).unwrap());

        // Add delivery fee if applicable
        let subtotal_with_delivery = self.subtotal.add(&self.delivery_fee).unwrap();

        // Tax will be calculated by the tax engine
        self.total_amount = subtotal_with_delivery.add(&self.tax_amount).unwrap();
        self.updated_at = Utc::now();
    }

    pub fn close(&mut self) -> Result<(), String> {
        match self.status {
            OrderStatus::Open => {
                self.status = OrderStatus::Closed;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!("Cannot close order in status: {:?}", self.status)),
        }
    }

    pub fn cancel(&mut self) -> Result<(), String> {
        match self.status {
            OrderStatus::Open => {
                self.status = OrderStatus::Cancelled;
                self.updated_at = Utc::now();
                Ok(())
            }
            _ => Err(format!("Cannot cancel order in status: {:?}", self.status)),
        }
    }
}

/// Menu item category
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct MenuItemCategory {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Menu item
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct MenuItem {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price: Money,
    pub tax_rate: rust_decimal::Decimal, // Tax rate as percentage (e.g., 8.5 for 8.5%)
    pub is_available: bool,
    pub preparation_time_minutes: Option<i32>,
    pub display_order: i32,
    pub image_url: Option<String>, // Product image URL
    pub stock_quantity: i32, // Current stock quantity
    pub low_stock_threshold: i32, // Alert when stock falls below this
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MenuItem {
    pub fn new(
        category_id: Uuid,
        name: String,
        price: Money,
        tax_rate: rust_decimal::Decimal,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            category_id,
            name,
            description: None,
            price,
            tax_rate,
            is_available: true,
            preparation_time_minutes: None,
            display_order: 0,
            image_url: None,
            stock_quantity: 0,
            low_stock_threshold: 10,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn calculate_tax(&self, quantity: i32) -> Money {
        let subtotal = self.price.multiply(quantity.into()).unwrap();
        subtotal.percentage(self.tax_rate).unwrap()
    }

    /// Check if the item is in stock
    pub fn in_stock(&self) -> bool {
        self.stock_quantity > 0
    }

    /// Check if the item is low on stock
    pub fn is_low_stock(&self) -> bool {
        self.stock_quantity <= self.low_stock_threshold
    }

    /// Adjust stock quantity
    pub fn adjust_stock(&mut self, quantity: i32) -> Result<(), String> {
        let new_quantity = self.stock_quantity + quantity;
        if new_quantity < 0 {
            return Err("Insufficient stock".to_string());
        }
        self.stock_quantity = new_quantity;
        self.updated_at = Utc::now();
        Ok(())
    }

    /// Set stock quantity directly
    pub fn set_stock(&mut self, quantity: i32) {
        self.stock_quantity = quantity;
        self.updated_at = Utc::now();
    }
}

/// Staff member
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Staff {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: StaffRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Staff role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum StaffRole {
    Server,
    Manager,
    Kitchen,
    Host,
    Admin,
}

/// Payment method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum PaymentMethod {
    Cash,
    Card,
    MobileMoney,
    Mpesa,
}

/// Payment status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}

/// Payment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Payment {
    pub id: Uuid,
    pub order_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub method: PaymentMethod,
    pub status: PaymentStatus,
    pub transaction_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Inventory transaction type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum InventoryTransactionType {
    StockIn,   // Adding stock
    StockOut,  // Removing stock (sale, waste, etc.)
    Adjustment, // Manual adjustment
    Transfer,  // Transfer between locations
}

/// Inventory transaction
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct InventoryTransaction {
    pub id: Uuid,
    pub menu_item_id: Uuid,
    pub transaction_type: InventoryTransactionType,
    pub quantity: i32,
    pub notes: Option<String>,
    pub created_by: Uuid, // Staff member who created the transaction
    pub created_at: DateTime<Utc>,
}

/// Registry entry for tracking system events
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct RegistryEntry {
    pub id: Uuid,
    pub entity_type: String, // e.g., "order", "table", "menu_item", "inventory"
    pub entity_id: Uuid,
    pub action: String, // e.g., "created", "updated", "deleted", "stock_adjusted"
    pub details: Option<String>, // JSON string with additional details
    pub created_by: Uuid, // Staff member who performed the action
    pub created_at: DateTime<Utc>,
}

/// Order type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum OrderType {
    DineIn,
    Takeout,
    Delivery,
}

/// Preparation status for order items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum PreparationStatus {
    Pending,
    Preparing,
    Ready,
    Served,
}

/// Customer
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Customer {
    pub id: Uuid,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub loyalty_points: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Reservation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum ReservationStatus {
    Confirmed,
    Seated,
    Cancelled,
    NoShow,
}

/// Reservation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Reservation {
    pub id: Uuid,
    pub table_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub reservation_time: DateTime<Utc>,
    pub party_size: i32,
    pub status: ReservationStatus,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Bill split
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct BillSplit {
    pub id: Uuid,
    pub order_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub amount: Money,
    pub created_at: DateTime<Utc>,
}

/// Item modification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct ItemModification {
    pub id: Uuid,
    pub order_item_id: Uuid,
    pub modification: String,
    pub price_adjustment: Money,
    pub created_at: DateTime<Utc>,
}

/// User role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum UserRole {
    Admin,
    Manager,
    Server,
    Kitchen,
    Host,
}

/// User for authentication
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub role: UserRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Discount type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(feature = "db", sqlx(type_name = "text"))]
pub enum DiscountType {
    Percentage,
    FixedAmount,
}

/// Discount
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Discount {
    pub id: Uuid,
    pub code: String,
    pub discount_type: DiscountType,
    pub value: Money,
    pub min_order_amount: Money,
    pub valid_from: DateTime<Utc>,
    pub valid_to: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Location for multi-location support
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Location {
    pub id: Uuid,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Payment {
    pub fn new(order_id: Uuid, amount: rust_decimal::Decimal, method: PaymentMethod) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            order_id,
            amount,
            method,
            status: PaymentStatus::Pending,
            transaction_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn complete(&mut self, transaction_id: String) {
        self.status = PaymentStatus::Completed;
        self.transaction_id = Some(transaction_id);
        self.updated_at = Utc::now();
    }

    pub fn fail(&mut self) {
        self.status = PaymentStatus::Failed;
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_table_state_machine() {
        let mut table = Table::new(1, 4, Some("Main Floor".to_string()));
        assert_eq!(table.status, TableStatus::Empty);

        // Occupy table
        assert!(table.occupy().is_ok());
        assert_eq!(table.status, TableStatus::Occupied);

        // Clear table
        assert!(table.clear().is_ok());
        assert_eq!(table.status, TableStatus::Dirty);

        // Clean table
        assert!(table.clean().is_ok());
        assert_eq!(table.status, TableStatus::Empty);

        // Try invalid transitions
        assert!(table.clear().is_err());
        assert!(table.clean().is_err());
    }

    #[test]
    fn test_order_item_lifecycle() {
        let order_id = Uuid::new_v4();
        let menu_item_id = Uuid::new_v4();
        let price = Money::new(dec!(10.50), "USD").unwrap();
        
        let mut item = OrderItem::new(order_id, menu_item_id, 2, price);
        assert_eq!(item.status, OrderItemStatus::Pending);
        assert_eq!(item.subtotal().amount(), dec!(21.00));

        // Fire item
        assert!(item.fire().is_ok());
        assert_eq!(item.status, OrderItemStatus::Fired);

        // Void item
        assert!(item.void("Customer request".to_string(), Uuid::new_v4()).is_ok());
        assert_eq!(item.status, OrderItemStatus::Voided);
        assert_eq!(item.void_reason, Some("Customer request".to_string()));
    }

    #[test]
    fn test_menu_item_tax_calculation() {
        let category_id = Uuid::new_v4();
        let price = Money::new(dec!(20.00), "USD").unwrap();
        let menu_item = MenuItem::new(category_id, "Test Item".to_string(), price, dec!(8.5));
        
        let tax = menu_item.calculate_tax(1);
        assert_eq!(tax.amount(), dec!(1.70)); // 8.5% of $20.00
    }
}
