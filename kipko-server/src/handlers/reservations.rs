//! Reservation management handlers

use crate::{AppState, ApiResponse};
use actix_web::{web, HttpResponse, Result};
use tracing::error;
use serde::Deserialize;
use sqlx::Row;
use uuid::Uuid;
use kipko_core::{Reservation, ReservationStatus};

#[derive(Debug, Deserialize)]
pub struct CreateReservationRequest {
    pub table_id: Uuid,
    pub customer_id: Option<Uuid>,
    pub reservation_time: chrono::DateTime<chrono::Utc>,
    pub party_size: i32,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateReservationRequest {
    pub status: Option<String>,
    pub party_size: Option<i32>,
    pub notes: Option<String>,
}

pub async fn get_reservations(
    state: web::Data<AppState>,
) -> Result<HttpResponse> {
    let rows = sqlx::query(
        r#"
        SELECT id, table_id, customer_id, reservation_time, party_size, status, notes, created_at, updated_at
        FROM reservations
        ORDER BY reservation_time ASC
        "#
    )
    .fetch_all(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch reservations: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch reservations"}))
    })?;

    let reservations: Vec<Reservation> = rows.into_iter().map(|row| Reservation {
        id: row.get("id"),
        table_id: row.get("table_id"),
        customer_id: row.get("customer_id"),
        reservation_time: row.get("reservation_time"),
        party_size: row.get("party_size"),
        status: match row.get::<&str, _>("status") {
            "Confirmed" => ReservationStatus::Confirmed,
            "Seated" => ReservationStatus::Seated,
            "Cancelled" => ReservationStatus::Cancelled,
            "NoShow" => ReservationStatus::NoShow,
            _ => ReservationStatus::Confirmed,
        },
        notes: row.get("notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }).collect();

    Ok(HttpResponse::Ok().json(ApiResponse::success(reservations)))
}

pub async fn get_reservation(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<HttpResponse> {
    let id = path.into_inner();

    let row = sqlx::query(
        r#"
        SELECT id, table_id, customer_id, reservation_time, party_size, status, notes, created_at, updated_at
        FROM reservations
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch reservation: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to fetch reservation"}))
    })?;

    match row {
        Some(row) => {
            let reservation = Reservation {
                id: row.get("id"),
                table_id: row.get("table_id"),
                customer_id: row.get("customer_id"),
                reservation_time: row.get("reservation_time"),
                party_size: row.get("party_size"),
                status: match row.get::<&str, _>("status") {
                    "Confirmed" => ReservationStatus::Confirmed,
                    "Seated" => ReservationStatus::Seated,
                    "Cancelled" => ReservationStatus::Cancelled,
                    "NoShow" => ReservationStatus::NoShow,
                    _ => ReservationStatus::Confirmed,
                },
                notes: row.get("notes"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(HttpResponse::Ok().json(ApiResponse::success(reservation)))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Reservation not found"}))),
    }
}

pub async fn create_reservation(
    state: web::Data<AppState>,
    request: web::Json<CreateReservationRequest>,
) -> Result<HttpResponse> {
    let row = sqlx::query(
        r#"
        INSERT INTO reservations (table_id, customer_id, reservation_time, party_size, status, notes)
        VALUES ($1, $2, $3, $4, 'Confirmed', $5)
        RETURNING id, table_id, customer_id, reservation_time, party_size, status, notes, created_at, updated_at
        "#
    )
    .bind(request.table_id)
    .bind(request.customer_id)
    .bind(request.reservation_time)
    .bind(request.party_size)
    .bind(&request.notes)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to create reservation: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to create reservation"}))
    })?;

    let reservation = Reservation {
        id: row.get("id"),
        table_id: row.get("table_id"),
        customer_id: row.get("customer_id"),
        reservation_time: row.get("reservation_time"),
        party_size: row.get("party_size"),
        status: ReservationStatus::Confirmed,
        notes: row.get("notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(reservation)))
}

pub async fn update_reservation(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    request: web::Json<UpdateReservationRequest>,
) -> Result<HttpResponse> {
    let id = path.into_inner();

    let row = sqlx::query(
        r#"
        UPDATE reservations
        SET
            status = COALESCE($2, status),
            party_size = COALESCE($3, party_size),
            notes = COALESCE($4, notes),
            updated_at = CURRENT_TIMESTAMP
        WHERE id = $1
        RETURNING id, table_id, customer_id, reservation_time, party_size, status, notes, created_at, updated_at
        "#
    )
    .bind(id)
    .bind(&request.status)
    .bind(request.party_size)
    .bind(&request.notes)
    .fetch_optional(&state.db_pool)
    .await
    .map_err(|e| {
        error!("Failed to update reservation: {}", e);
        return HttpResponse::InternalServerError().json(serde_json::json!({"error": "Failed to update reservation"}))
    })?;

    match row {
        Some(row) => {
            let reservation = Reservation {
                id: row.get("id"),
                table_id: row.get("table_id"),
                customer_id: row.get("customer_id"),
                reservation_time: row.get("reservation_time"),
                party_size: row.get("party_size"),
                status: match row.get::<&str, _>("status") {
                    "Confirmed" => ReservationStatus::Confirmed,
                    "Seated" => ReservationStatus::Seated,
                    "Cancelled" => ReservationStatus::Cancelled,
                    "NoShow" => ReservationStatus::NoShow,
                    _ => ReservationStatus::Confirmed,
                },
                notes: row.get("notes"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            };
            Ok(HttpResponse::Ok().json(ApiResponse::success(reservation)))
        }
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({"error": "Reservation not found"}))),
    }
}
