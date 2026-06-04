//! Kipko POS Server
//!
//! This is the main server application for the Kipko Point of Sale system.
//! It provides REST API endpoints for managing restaurant operations using Actix-web.

use actix_web::{web, App, HttpServer, HttpResponse, Responder, Result};
use actix_cors::Cors;
use actix_web::middleware::Logger;
use sqlx::postgres::PgPoolOptions;
use env_logger::Env;

mod handlers;
mod models;
mod database;
mod services;

use handlers::*;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: sqlx::PgPool,
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    env_logger::init_from_env(Env::default().default_filter_or("info"));

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
    let app_state = web::Data::new(AppState { db_pool });

    // Get server configuration
    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .expect("Invalid port number");

    let bind_address = format!("{}:{}", host, port);
    tracing::info!("Starting Kipko POS server on {}", bind_address);

    // Start the server
    HttpServer::new(move || {
        let cors = Cors::permissive();

        App::new()
            .app_data(app_state.clone())
            .wrap(cors)
            .wrap(Logger::default())
            // Health check endpoint
            .route("/health", web::get().to(health_check))
            // Tables endpoints
            .service(
                web::scope("/tables")
                    .route("", web::get().to(get_tables))
                    .route("", web::post().to(create_table))
                    .route("/{id}", web::get().to(get_table))
                    .route("/{id}", web::put().to(update_table))
                    .route("/{id}", web::delete().to(delete_table))
                    .route("/{id}/occupy", web::post().to(occupy_table))
                    .route("/{id}/clear", web::post().to(clear_table))
                    .route("/{id}/clean", web::post().to(clean_table))
            )
            // Menu items endpoints
            .service(
                web::scope("/menu")
                    .route("/categories", web::get().to(get_menu_categories))
                    .route("/items", web::get().to(get_menu_items))
                    .route("/items", web::post().to(create_menu_item))
                    .route("/items/{id}", web::get().to(get_menu_item))
                    .route("/items/{id}", web::put().to(update_menu_item))
                    .route("/items/{id}", web::delete().to(delete_menu_item))
            )
            // Orders endpoints
            .service(
                web::scope("/orders")
                    .route("", web::get().to(get_orders))
                    .route("", web::post().to(create_order))
                    .route("/{id}", web::get().to(get_order))
                    .route("/{id}", web::put().to(update_order))
                    .route("/{id}", web::delete().to(delete_order))
                    .route("/{id}/items", web::get().to(handlers::order_items::get_order_items))
                    .route("/{id}/items", web::post().to(handlers::order_items::add_order_item))
                    .route("/{id}/items/{item_id}", web::put().to(handlers::order_items::update_order_item))
                    .route("/{id}/items/{item_id}", web::delete().to(handlers::order_items::delete_order_item))
                    .route("/{id}/calculate-tax", web::post().to(calculate_order_tax))
                    .route("/{id}/close", web::post().to(close_order))
            )
            // Payments endpoints
            .service(
                web::scope("/payments")
                    .route("", web::get().to(get_payments))
                    .route("", web::post().to(create_payment))
                    .route("/{id}", web::get().to(get_payment))
                    .route("/{id}/complete", web::post().to(handlers::payments::complete_payment))
            )
            .service(
                web::scope("/orders")
                    .route("/{order_id}/payments", web::get().to(handlers::payments::get_order_payments))
            )
            // Customers endpoints
            .service(
                web::scope("/customers")
                    .route("", web::get().to(handlers::customers::get_customers))
                    .route("", web::post().to(handlers::customers::create_customer))
                    .route("/{id}", web::get().to(handlers::customers::get_customer))
                    .route("/{id}", web::put().to(handlers::customers::update_customer))
            )
            // Reservations endpoints
            .service(
                web::scope("/reservations")
                    .route("", web::get().to(handlers::reservations::get_reservations))
                    .route("", web::post().to(handlers::reservations::create_reservation))
                    .route("/{id}", web::get().to(handlers::reservations::get_reservation))
                    .route("/{id}", web::put().to(handlers::reservations::update_reservation))
            )
            // Staff endpoints
            .service(
                web::scope("/staff")
                    .route("", web::get().to(get_staff))
                    .route("", web::post().to(create_staff))
                    .route("/{id}", web::get().to(get_staff_member))
                    .route("/{id}", web::put().to(update_staff))
                    .route("/{id}", web::delete().to(delete_staff))
            )
            // Accounting endpoints
            .service(
                web::scope("/accounting")
                    .route("/transactions", web::get().to(get_transactions))
                    .route("/accounts", web::get().to(get_accounts))
                    .route("/balances", web::get().to(get_account_balances))
                    .route("/trial-balance", web::get().to(handlers::accounting::get_trial_balance))
                    .route("/income-statement", web::get().to(handlers::accounting::get_income_statement))
                    .route("/balance-sheet", web::get().to(handlers::accounting::get_balance_sheet))
                    .route("/periods", web::get().to(handlers::accounting::get_accounting_periods))
                    .route("/periods/{id}/close", web::post().to(handlers::accounting::close_accounting_period))
                    .route("/reconciliations/{account_id}", web::get().to(handlers::accounting::get_bank_reconciliations))
                    .route("/reconciliations", web::post().to(handlers::accounting::create_bank_reconciliation))
            )
            // Tax endpoints
            .service(
                web::scope("/tax")
                    .route("/jurisdictions", web::get().to(get_tax_jurisdictions))
                    .route("/exemptions", web::get().to(get_tax_exemptions))
            )
            // Inventory endpoints
            .service(
                web::scope("/inventory")
                    .route("/transactions", web::get().to(get_inventory_transactions))
                    .route("/transactions", web::post().to(create_inventory_transaction))
                    .route("/transactions/item/{menu_item_id}", web::get().to(get_inventory_transactions_for_item))
            )
            // Registry endpoints
            .service(
                web::scope("/registry")
                    .route("/entries", web::get().to(get_registry_entries))
                    .route("/entries", web::post().to(create_registry_entry))
                    .route("/entries/{entity_type}/{entity_id}", web::get().to(get_registry_entries_for_entity))
            )
    })
    .bind(&bind_address)?
    .run()
    .await?;

    Ok(())
}

/// Health check endpoint
async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "kipko-pos-server",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}