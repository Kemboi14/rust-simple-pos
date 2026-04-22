//! API Service
//! 
//! This service handles communication with the kipko-server REST API.

use kipko_core::{Table, Order, OrderItem, MenuItem, MenuItemCategory, Staff, InventoryTransaction, RegistryEntry, Payment, Customer, Reservation};
use wasm_bindgen::JsCast;
use web_sys::{Request, RequestInit, RequestMode, Response, Headers};

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
                .unwrap_or_else(|_| "http://localhost:3000".to_string()),
        }
    }

    async fn fetch(&self, url: &str, method: &str, body: Option<String>) -> Result<serde_json::Value, String> {
        let mut opts = RequestInit::new();
        opts.method(method);
        opts.mode(RequestMode::Cors);
        
        if let Some(b) = body {
            opts.body(Some(&b.into()));
        }

        let headers = Headers::new().map_err(|e| format!("Failed to create headers: {:?}", e))?;
        headers.set("Content-Type", "application/json").map_err(|e| format!("Failed to set headers: {:?}", e))?;
        opts.headers(&headers);

        let request = Request::new_with_str_and_init(url, &opts)
            .map_err(|e| format!("Failed to create request: {:?}", e))?;

        let window = web_sys::window().ok_or("No window")?;
        let response_promise = window.fetch_with_request(&request);
        let response = wasm_bindgen_futures::JsFuture::from(response_promise)
            .await
            .map_err(|e| format!("Fetch failed: {:?}", e))?;

        let response: Response = response.dyn_into().map_err(|e| format!("Invalid response: {:?}", e))?;
        
        if !response.ok() {
            return Err(format!("API error: {}", response.status()));
        }

        let text_promise = response.text().map_err(|e| format!("Failed to get text promise: {:?}", e))?;
        let text = wasm_bindgen_futures::JsFuture::from(text_promise)
            .await
            .map_err(|e| format!("Failed to get response text: {:?}", e))?;
        
        let text_str = text.as_string().ok_or("Response is not a string")?;
        
        serde_json::from_str(&text_str).map_err(|e| format!("Failed to parse JSON: {}", e))
    }

    // Tables API
    pub async fn get_tables(&self) -> Result<Vec<Table>, String> {
        let url = format!("{}/tables", self.base_url);
        let api_response = self.fetch(&url, "GET", None).await?;

        let tables_data = api_response["data"]
            .as_array()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let tables: Vec<Table> = serde_json::from_value(serde_json::Value::Array(tables_data.clone()))
            .map_err(|e| format!("Failed to deserialize tables: {}", e))?;

        Ok(tables)
    }

    // Orders API
    pub async fn get_orders(&self) -> Result<Vec<Order>, String> {
        let url = format!("{}/orders", self.base_url);
        let api_response = self.fetch(&url, "GET", None).await?;

        let orders_data = api_response["data"]
            .as_array()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let orders: Vec<Order> = serde_json::from_value(serde_json::Value::Array(orders_data.clone()))
            .map_err(|e| format!("Failed to deserialize orders: {}", e))?;

        Ok(orders)
    }

    pub async fn create_order(&self, order_data: crate::pages::orders::CreateOrderData) -> Result<Order, String> {
        let url = format!("{}/orders", self.base_url);
        let body = serde_json::json!({
            "table_id": order_data.table_id,
            "staff_id": order_data.staff_id
        });
        let api_response = self.fetch(&url, "POST", Some(body.to_string())).await?;

        let order_data = api_response["data"]
            .as_object()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let order: Order = serde_json::from_value(serde_json::Value::Object(order_data.clone()))
            .map_err(|e| format!("Failed to deserialize order: {}", e))?;

        Ok(order)
    }

    pub async fn create_table(&self, table_data: crate::pages::floorplan::CreateTableData) -> Result<Table, String> {
        let url = format!("{}/tables", self.base_url);
        let body = serde_json::json!({
            "number": table_data.number,
            "capacity": table_data.capacity,
            "location": table_data.location
        });
        let api_response = self.fetch(&url, "POST", Some(body.to_string())).await?;

        let table_data = api_response["data"]
            .as_object()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let table: Table = serde_json::from_value(serde_json::Value::Object(table_data.clone()))
            .map_err(|e| format!("Failed to deserialize table: {}", e))?;

        Ok(table)
    }

    pub async fn occupy_table(&self, table_id: uuid::Uuid) -> Result<(), String> {
        let url = format!("{}/tables/{}/occupy", self.base_url, table_id);
        self.fetch(&url, "POST", None).await?;
        Ok(())
    }

    pub async fn clear_table(&self, table_id: uuid::Uuid) -> Result<(), String> {
        let url = format!("{}/tables/{}/clear", self.base_url, table_id);
        self.fetch(&url, "POST", None).await?;
        Ok(())
    }

    pub async fn clean_table(&self, table_id: uuid::Uuid) -> Result<(), String> {
        let url = format!("{}/tables/{}/clean", self.base_url, table_id);
        self.fetch(&url, "POST", None).await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn close_order(&self, order_id: uuid::Uuid) -> Result<(), String> {
        let url = format!("{}/orders/{}/close", self.base_url, order_id);
        self.fetch(&url, "POST", None).await?;
        Ok(())
    }

    // Menu API
    pub async fn get_menu_categories(&self) -> Result<Vec<MenuItemCategory>, String> {
        let url = format!("{}/menu/categories", self.base_url);
        let api_response = self.fetch(&url, "GET", None).await?;

        let categories_data = api_response["data"]
            .as_array()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let categories: Vec<MenuItemCategory> = serde_json::from_value(serde_json::Value::Array(categories_data.clone()))
            .map_err(|e| format!("Failed to deserialize categories: {}", e))?;

        Ok(categories)
    }

    pub async fn create_menu_item(&self, item_data: crate::pages::menu::CreateMenuItemData) -> Result<MenuItem, String> {
        let url = format!("{}/menu/items", self.base_url);
        let body = serde_json::json!({
            "category_id": item_data.category_id,
            "name": item_data.name,
            "description": item_data.description,
            "price": item_data.price,
            "tax_rate": item_data.tax_rate,
            "image_url": item_data.image_url,
            "stock_quantity": item_data.stock_quantity,
            "low_stock_threshold": item_data.low_stock_threshold
        });
        let api_response = self.fetch(&url, "POST", Some(body.to_string())).await?;

        let item_data = api_response["data"]
            .as_object()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let item: MenuItem = serde_json::from_value(serde_json::Value::Object(item_data.clone()))
            .map_err(|e| format!("Failed to deserialize menu item: {}", e))?;

        Ok(item)
    }

    pub async fn get_menu_items(&self) -> Result<Vec<MenuItem>, String> {
        let url = format!("{}/menu/items", self.base_url);
        let api_response = self.fetch(&url, "GET", None).await?;

        let items_data = api_response["data"]
            .as_array()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let items: Vec<MenuItem> = serde_json::from_value(serde_json::Value::Array(items_data.clone()))
            .map_err(|e| format!("Failed to deserialize menu items: {}", e))?;

        Ok(items)
    }

    // Inventory API
    pub async fn get_inventory_transactions(&self) -> Result<Vec<InventoryTransaction>, String> {
        let url = format!("{}/inventory/transactions", self.base_url);
        let api_response = self.fetch(&url, "GET", None).await?;

        let trans_data = api_response["data"]
            .as_array()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let transactions: Vec<InventoryTransaction> = serde_json::from_value(serde_json::Value::Array(trans_data.clone()))
            .map_err(|e| format!("Failed to deserialize inventory transactions: {}", e))?;

        Ok(transactions)
    }

    pub async fn create_inventory_transaction(
        &self,
        menu_item_id: uuid::Uuid,
        transaction_type: String,
        quantity: i32,
        notes: String,
        created_by: uuid::Uuid,
    ) -> Result<(), String> {
        let url = format!("{}/inventory/transactions", self.base_url);
        let body = serde_json::json!({
            "menu_item_id": menu_item_id,
            "transaction_type": transaction_type,
            "quantity": quantity,
            "notes": notes,
            "created_by": created_by
        });
        self.fetch(&url, "POST", Some(body.to_string())).await?;
        Ok(())
    }

    // Registry API
    pub async fn get_registry_entries(&self) -> Result<Vec<RegistryEntry>, String> {
        let url = format!("{}/registry/entries", self.base_url);
        let api_response = self.fetch(&url, "GET", None).await?;

        let entries_data = api_response["data"]
            .as_array()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let entries: Vec<RegistryEntry> = serde_json::from_value(serde_json::Value::Array(entries_data.clone()))
            .map_err(|e| format!("Failed to deserialize registry entries: {}", e))?;

        Ok(entries)
    }

    // Order Items API
    pub async fn get_order_items(&self, order_id: uuid::Uuid) -> Result<Vec<OrderItem>, String> {
        let url = format!("{}/orders/{}/items", self.base_url, order_id);
        let api_response = self.fetch(&url, "GET", None).await?;

        let items_data = api_response["data"]
            .as_array()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let items: Vec<OrderItem> = serde_json::from_value(serde_json::Value::Array(items_data.clone()))
            .map_err(|e| format!("Failed to deserialize order items: {}", e))?;

        Ok(items)
    }

    pub async fn add_order_item(&self, order_id: uuid::Uuid, menu_item_id: uuid::Uuid, quantity: i32, notes: Option<String>) -> Result<OrderItem, String> {
        let url = format!("{}/orders/{}/items", self.base_url, order_id);
        let body = serde_json::json!({
            "menu_item_id": menu_item_id,
            "quantity": quantity,
            "notes": notes
        });
        let api_response = self.fetch(&url, "POST", Some(body.to_string())).await?;

        let item_data = api_response["data"]
            .as_object()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let item: OrderItem = serde_json::from_value(serde_json::Value::Object(item_data.clone()))
            .map_err(|e| format!("Failed to deserialize order item: {}", e))?;

        Ok(item)
    }

    pub async fn update_order_item(&self, order_id: uuid::Uuid, item_id: uuid::Uuid, quantity: Option<i32>, preparation_status: Option<String>) -> Result<OrderItem, String> {
        let url = format!("{}/orders/{}/items/{}", self.base_url, order_id, item_id);
        let body = serde_json::json!({
            "quantity": quantity,
            "preparation_status": preparation_status
        });
        let api_response = self.fetch(&url, "PUT", Some(body.to_string())).await?;

        let item_data = api_response["data"]
            .as_object()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let item: OrderItem = serde_json::from_value(serde_json::Value::Object(item_data.clone()))
            .map_err(|e| format!("Failed to deserialize order item: {}", e))?;

        Ok(item)
    }

    pub async fn delete_order_item(&self, order_id: uuid::Uuid, item_id: uuid::Uuid) -> Result<(), String> {
        let url = format!("{}/orders/{}/items/{}", self.base_url, order_id, item_id);
        self.fetch(&url, "DELETE", None).await?;
        Ok(())
    }

    // Payments API
    pub async fn get_order_payments(&self, order_id: uuid::Uuid) -> Result<Vec<Payment>, String> {
        let url = format!("{}/orders/{}/payments", self.base_url, order_id);
        let api_response = self.fetch(&url, "GET", None).await?;

        let payments_data = api_response["data"]
            .as_array()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let payments: Vec<Payment> = serde_json::from_value(serde_json::Value::Array(payments_data.clone()))
            .map_err(|e| format!("Failed to deserialize payments: {}", e))?;

        Ok(payments)
    }

    pub async fn create_payment(&self, order_id: uuid::Uuid, amount: rust_decimal::Decimal, method: String) -> Result<Payment, String> {
        let url = format!("{}/payments", self.base_url);
        let body = serde_json::json!({
            "order_id": order_id,
            "amount": amount,
            "method": method
        });
        let api_response = self.fetch(&url, "POST", Some(body.to_string())).await?;

        let payment_data = api_response["data"]
            .as_object()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let payment: Payment = serde_json::from_value(serde_json::Value::Object(payment_data.clone()))
            .map_err(|e| format!("Failed to deserialize payment: {}", e))?;

        Ok(payment)
    }

    pub async fn complete_payment(&self, payment_id: uuid::Uuid, transaction_id: String) -> Result<Payment, String> {
        let url = format!("{}/payments/{}/complete", self.base_url, payment_id);
        let body = serde_json::json!({ "transaction_id": transaction_id });
        let api_response = self.fetch(&url, "POST", Some(body.to_string())).await?;

        let payment_data = api_response["data"]
            .as_object()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let payment: Payment = serde_json::from_value(serde_json::Value::Object(payment_data.clone()))
            .map_err(|e| format!("Failed to deserialize payment: {}", e))?;

        Ok(payment)
    }

    // Customers API
    pub async fn get_customers(&self) -> Result<Vec<Customer>, String> {
        let url = format!("{}/customers", self.base_url);
        let api_response = self.fetch(&url, "GET", None).await?;

        let customers_data = api_response["data"]
            .as_array()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let customers: Vec<Customer> = serde_json::from_value(serde_json::Value::Array(customers_data.clone()))
            .map_err(|e| format!("Failed to deserialize customers: {}", e))?;

        Ok(customers)
    }

    pub async fn create_customer(&self, name: String, phone: Option<String>, email: Option<String>) -> Result<Customer, String> {
        let url = format!("{}/customers", self.base_url);
        let body = serde_json::json!({
            "name": name,
            "phone": phone,
            "email": email
        });
        let api_response = self.fetch(&url, "POST", Some(body.to_string())).await?;

        let customer_data = api_response["data"]
            .as_object()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let customer: Customer = serde_json::from_value(serde_json::Value::Object(customer_data.clone()))
            .map_err(|e| format!("Failed to deserialize customer: {}", e))?;

        Ok(customer)
    }

    // Reservations API
    pub async fn get_reservations(&self) -> Result<Vec<Reservation>, String> {
        let url = format!("{}/reservations", self.base_url);
        let api_response = self.fetch(&url, "GET", None).await?;

        let reservations_data = api_response["data"]
            .as_array()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let reservations: Vec<Reservation> = serde_json::from_value(serde_json::Value::Array(reservations_data.clone()))
            .map_err(|e| format!("Failed to deserialize reservations: {}", e))?;

        Ok(reservations)
    }

    pub async fn create_reservation(&self, table_id: uuid::Uuid, customer_id: Option<uuid::Uuid>, reservation_time: chrono::DateTime<chrono::Utc>, party_size: i32, notes: Option<String>) -> Result<Reservation, String> {
        let url = format!("{}/reservations", self.base_url);
        let body = serde_json::json!({
            "table_id": table_id,
            "customer_id": customer_id,
            "reservation_time": reservation_time.to_rfc3339(),
            "party_size": party_size,
            "notes": notes
        });
        let api_response = self.fetch(&url, "POST", Some(body.to_string())).await?;

        let reservation_data = api_response["data"]
            .as_object()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let reservation: Reservation = serde_json::from_value(serde_json::Value::Object(reservation_data.clone()))
            .map_err(|e| format!("Failed to deserialize reservation: {}", e))?;

        Ok(reservation)
    }

    // Staff API
    pub async fn get_staff(&self) -> Result<Vec<Staff>, String> {
        let url = format!("{}/staff", self.base_url);
        let api_response = self.fetch(&url, "GET", None).await?;

        let staff_data = api_response["data"]
            .as_array()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let staff: Vec<Staff> = serde_json::from_value(serde_json::Value::Array(staff_data.clone()))
            .map_err(|e| format!("Failed to deserialize staff: {}", e))?;

        Ok(staff)
    }

    pub async fn create_staff(&self, name: String, email: String, role: String) -> Result<Staff, String> {
        let url = format!("{}/staff", self.base_url);
        let body = serde_json::json!({
            "name": name,
            "email": email,
            "role": role
        });
        let api_response = self.fetch(&url, "POST", Some(body.to_string())).await?;

        let staff_data = api_response["data"]
            .as_object()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let staff: Staff = serde_json::from_value(serde_json::Value::Object(staff_data.clone()))
            .map_err(|e| format!("Failed to deserialize staff: {}", e))?;

        Ok(staff)
    }
}
