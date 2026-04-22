//! Kipko POS Server
//! 
//! This is the main server application for the Kipko Point of Sale system.
//! It provides REST API endpoints for managing restaurant operations.

use axum::{
    routing::{get, post, put},
    Router,
};
use axum::http::StatusCode;
use axum::response::Json;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

mod handlers;
mod models;
mod database;

use handlers::*;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    // Create database connection pool
    let db_pool = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("Failed to connect to database");

    // Run database migrations
    sqlx::migrate!("../migrations")
        .run(&db_pool)
        .await
        .expect("Failed to run database migrations");

    // Create application state
    let app_state = AppState { db_pool };

    // Build the application router
    let app = Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        
        // Tables endpoints
        .route("/tables", get(get_tables).post(create_table))
        .route("/tables/:id", get(get_table).put(update_table).delete(delete_table))
        .route("/tables/:id/occupy", post(occupy_table))
        .route("/tables/:id/clear", post(clear_table))
        .route("/tables/:id/clean", post(clean_table))
        
        // Menu items endpoints
        .route("/menu/categories", get(get_menu_categories))
        .route("/menu/items", get(get_menu_items).post(create_menu_item))
        .route("/menu/items/:id", get(get_menu_item).put(update_menu_item).delete(delete_menu_item))
        
        // Orders endpoints
        .route("/orders", get(get_orders).post(create_order))
        .route("/orders/:id", get(get_order).put(update_order).delete(delete_order))
        .route("/orders/:id/items", get(handlers::order_items::get_order_items).post(handlers::order_items::add_order_item))
        .route("/orders/:id/items/:item_id", put(handlers::order_items::update_order_item).delete(handlers::order_items::delete_order_item))
        .route("/orders/:id/calculate-tax", post(calculate_order_tax))
        .route("/orders/:id/close", post(close_order))

        // Payments endpoints
        .route("/payments", get(get_payments).post(create_payment))
        .route("/payments/:id", get(get_payment))
        .route("/payments/:id/complete", post(handlers::payments::complete_payment))
        .route("/orders/:order_id/payments", get(handlers::payments::get_order_payments))

        // Customers endpoints
        .route("/customers", get(handlers::customers::get_customers).post(handlers::customers::create_customer))
        .route("/customers/:id", get(handlers::customers::get_customer).put(handlers::customers::update_customer))

        // Reservations endpoints
        .route("/reservations", get(handlers::reservations::get_reservations).post(handlers::reservations::create_reservation))
        .route("/reservations/:id", get(handlers::reservations::get_reservation).put(handlers::reservations::update_reservation))
        
        // Staff endpoints
        .route("/staff", get(get_staff).post(create_staff))
        .route("/staff/:id", get(get_staff_member).put(update_staff).delete(delete_staff))
        
        // Accounting endpoints
        .route("/accounting/transactions", get(get_transactions))
        .route("/accounting/accounts", get(get_accounts))
        .route("/accounting/balances", get(get_account_balances))
        
        // Tax endpoints
        .route("/tax/jurisdictions", get(get_tax_jurisdictions))
        .route("/tax/exemptions", get(get_tax_exemptions))

        // Inventory endpoints
        .route("/inventory/transactions", get(get_inventory_transactions).post(create_inventory_transaction))
        .route("/inventory/transactions/item/:menu_item_id", get(get_inventory_transactions_for_item))

        // Registry endpoints
        .route("/registry/entries", get(get_registry_entries).post(create_registry_entry))
        .route("/registry/entries/:entity_type/:entity_id", get(get_registry_entries_for_entity))

        // CORS middleware
        .layer(
            ServiceBuilder::new()
                .layer(CorsLayer::permissive())
        )
        .with_state(app_state);

    // Get server configuration
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .expect("Invalid port number");

    let addr = SocketAddr::from((host.parse::<std::net::IpAddr>().unwrap(), port));

    tracing::info!("Starting Kipko POS server on {}", addr);

    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// Health check endpoint
async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "service": "kipko-pos-server",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
