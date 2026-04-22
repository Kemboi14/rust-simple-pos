//! Reservation management handlers

use crate::{AppState, ApiResponse};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
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
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Reservation>>>, StatusCode> {
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
        tracing::error!("Failed to fetch reservations: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
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

    Ok(Json(ApiResponse::success(reservations)))
}

pub async fn get_reservation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Reservation>>, StatusCode> {
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
        tracing::error!("Failed to fetch reservation: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
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
            Ok(Json(ApiResponse::success(reservation)))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn create_reservation(
    State(state): State<AppState>,
    Json(request): Json<CreateReservationRequest>,
) -> Result<Json<ApiResponse<Reservation>>, StatusCode> {
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
        tracing::error!("Failed to create reservation: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
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

    Ok(Json(ApiResponse::success(reservation)))
}

pub async fn update_reservation(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateReservationRequest>,
) -> Result<Json<ApiResponse<Reservation>>, StatusCode> {
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
        tracing::error!("Failed to update reservation: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
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
            Ok(Json(ApiResponse::success(reservation)))
        }
        None => Err(StatusCode::NOT_FOUND),
    }
}
