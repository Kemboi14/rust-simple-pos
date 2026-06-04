//! Tax management handlers

use crate::{AppState, ApiResponse};
use actix_web::{web, HttpResponse, Result};
use tracing::error;
use sqlx::Row;
use kipko_core::tax::*;

/// Get all tax jurisdictions
pub async fn get_tax_jurisdictions(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let rows = sqlx::query(
        r#"
        SELECT
            id, name, code, tax_rate, is_active,
            effective_date, expiry_date, created_at, updated_at
        FROM tax_jurisdictions
        WHERE is_active = true
        ORDER BY name
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch tax jurisdictions: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch tax jurisdictions"}))
    })?;
    let jurisdictions: Vec<TaxJurisdiction> = rows.into_iter().map(|row| TaxJurisdiction {
        id: row.get("id"),
        name: row.get("name"),
        code: row.get("code"),
        tax_rate: row.get("tax_rate"),
        is_active: row.get("is_active"),
        effective_date: row.get("effective_date"),
        expiry_date: row.get("expiry_date"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(jurisdictions)))
}

/// Get all tax exemptions
pub async fn get_tax_exemptions(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let rows = sqlx::query(
        r#"
        SELECT
            id, name, exemption_type, certificate_number, is_active,
            created_at, updated_at
        FROM tax_exemptions
        WHERE is_active = true
        ORDER BY name
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch tax exemptions: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch tax exemptions"}))
    })?;

    let exemptions: Vec<TaxExemption> = rows.into_iter().map(|row| TaxExemption {
        id: row.get("id"),
        name: row.get("name"),
        exemption_type: match row.get::<&str, _>("exemption_type") {
            "NonProfit" => TaxExemptionType::NonProfit,
            "Government" => TaxExemptionType::Government,
            "Resale" => TaxExemptionType::Resale,
            "Agricultural" => TaxExemptionType::Agricultural,
            "Manufacturing" => TaxExemptionType::Manufacturing,
            "Other" => TaxExemptionType::Other,
            _ => TaxExemptionType::Other,
        },
        certificate_number: row.get("certificate_number"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(exemptions)))
}
