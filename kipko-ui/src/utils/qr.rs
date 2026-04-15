//! QR Code Generation Utility
//! 
//! This module provides functions for generating QR codes as base64-encoded images
//! for display in the web UI.

use qrcode::QrCode;
use base64::{Engine as _, engine::general_purpose::STANDARD};

/// Generate a QR code for the given data and return it as a base64-encoded SVG data URL
pub fn generate_qr_code(data: &str, size: u32) -> Result<String, String> {
    // Create QR code
    let qr_code = QrCode::new(data)
        .map_err(|e| format!("Failed to create QR code: {}", e))?;
    
    // Convert to SVG string
    let svg_string = qr_code.render::<qrcode::render::svg::Color>()
        .min_dimensions(size, size)
        .max_dimensions(size, size)
        .build();
    
    // Encode to base64
    let base64_string = STANDARD.encode(svg_string.as_bytes());
    
    // Return as data URL
    Ok(format!("data:image/svg+xml;base64,{}", base64_string))
}

/// Generate a QR code for a table (for customer scanning)
pub fn generate_table_qr_code(table_id: uuid::Uuid, table_number: i32, base_url: &str) -> String {
    let url = format!("{}/table/{}", base_url, table_id);
    generate_qr_code(&url, 200).unwrap_or_else(|_| {
        // Fallback: generate QR code with table number only
        generate_qr_code(&format!("TABLE-{}", table_number), 200).unwrap_or_default()
    })
}

/// Generate a QR code for an order (for payment scanning)
pub fn generate_order_qr_code(order_id: uuid::Uuid, base_url: &str) -> String {
    let url = format!("{}/order/pay/{}", base_url, order_id);
    generate_qr_code(&url, 200).unwrap_or_else(|_| {
        // Fallback: generate QR code with order ID only
        generate_qr_code(&order_id.to_string(), 200).unwrap_or_default()
    })
}

/// Generate a QR code for a menu (for customers to view menu)
pub fn generate_menu_qr_code(base_url: &str) -> String {
    let url = format!("{}/menu", base_url);
    generate_qr_code(&url, 200).unwrap_or_else(|_| {
        // Fallback: generate QR code with menu text
        generate_qr_code("MENU", 200).unwrap_or_default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_qr_code() {
        let result = generate_qr_code("test data", 200);
        assert!(result.is_ok());
        let data_url = result.unwrap();
        assert!(data_url.starts_with("data:image/svg+xml;base64,"));
    }

    #[test]
    fn test_generate_table_qr_code() {
        let table_id = uuid::Uuid::new_v4();
        let result = generate_table_qr_code(table_id, 5, "https://kipko.com");
        assert!(!result.is_empty());
        assert!(result.starts_with("data:image/svg+xml;base64,"));
    }

    #[test]
    fn test_generate_order_qr_code() {
        let order_id = uuid::Uuid::new_v4();
        let result = generate_order_qr_code(order_id, "https://kipko.com");
        assert!(!result.is_empty());
        assert!(result.starts_with("data:image/svg+xml;base64,"));
    }
}
