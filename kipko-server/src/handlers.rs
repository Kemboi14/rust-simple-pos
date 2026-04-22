//! API handlers for Kipko POS
//!
//! This module contains all the HTTP request handlers for the REST API endpoints.

use serde::{Deserialize, Serialize};

pub mod tables;
pub mod menu;
pub mod orders;
pub mod order_items;
pub mod payments;
pub mod staff;
pub mod accounting;
pub mod tax;
pub mod inventory;
pub mod registry;
pub mod customers;
pub mod reservations;

// Re-export handlers for convenience
pub use tables::*;
pub use menu::*;
pub use orders::*;
pub use order_items::*;
pub use payments::*;
pub use staff::*;
pub use accounting::*;
pub use tax::*;
pub use inventory::*;
pub use registry::*;
pub use customers::*;
pub use reservations::*;

/// Generic API response wrapper
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    #[allow(dead_code)]
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// Pagination query parameters
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            limit: Some(50),
        }
    }
}

/// Paginated response
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
    pub total_pages: u32,
}

impl<T> PaginatedResponse<T> {
    #[allow(dead_code)]
    pub fn new(items: Vec<T>, total: i64, page: u32, limit: u32) -> Self {
        let total_pages = ((total as f64) / (limit as f64)).ceil() as u32;
        Self {
            items,
            total,
            page,
            limit,
            total_pages,
        }
    }
}

/// Error response
#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}
