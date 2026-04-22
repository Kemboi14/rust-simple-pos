//! Reservations Page

use dioxus::prelude::*;
use kipko_core::{Reservation, Table, Customer};
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Card, Badge, BadgeVariant};

#[component]
pub fn ReservationsPage() -> Element {
    let api = ApiService::new();
    let api_clone1 = api.clone();
    let mut reservations = use_resource(move || {
        let api_clone = api_clone1.clone();
        async move {
            api_clone.get_reservations().await.unwrap_or_default()
        }
    });
    let api_clone2 = api.clone();
    let mut tables = use_resource(move || {
        let api_clone = api_clone2.clone();
        async move {
            api_clone.get_tables().await.unwrap_or_default()
        }
    });
    let api_clone3 = api.clone();
    let mut customers = use_resource(move || {
        let api_clone = api_clone3.clone();
        async move {
            api_clone.get_customers().await.unwrap_or_default()
        }
    });
    let mut show_add_reservation = use_signal(|| false);
    let mut selected_table = use_signal(|| Option::<uuid::Uuid>::None);
    let mut selected_customer = use_signal(|| Option::<uuid::Uuid>::None);
    let mut reservation_time = use_signal(|| String::new());
    let mut party_size = use_signal(|| 2);
    let mut notes = use_signal(|| String::new());

    rsx! {
        div { class: "p-6 space-y-6",
            // Header
            div { class: "flex justify-between items-center",
                h1 { class: "text-3xl font-bold text-gray-900", "Reservations" }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| show_add_reservation.set(true),
                    children: rsx! { "Add Reservation" }
                }
            }

            // Refresh Button
            Button {
                variant: ButtonVariant::Outline,
                onclick: move |_| {
                    reservations.restart();
                    tables.restart();
                    customers.restart();
                },
                children: rsx! { "Refresh" }
            }

            // Reservations List
            {let data = reservations.read();
            if let Some(reservation_list) = data.as_ref() {
                if reservation_list.is_empty() {
                    rsx! {
                        div { class: "text-center py-12",
                            p { class: "text-gray-500 text-lg", "No reservations yet" }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| show_add_reservation.set(true),
                                children: rsx! { "Add First Reservation" }
                            }
                        }
                    }
                } else {
                    rsx! {
                        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                            for reservation in reservation_list {
                                ReservationCard {
                                    reservation: reservation.clone()
                                }
                            }
                        }
                    }
                }
            } else {
                rsx! {
                    div { class: "flex justify-center py-12",
                        div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-[#e0311f]" }
                    }
                }
            }}

            // Add Reservation Modal
            {let show = *show_add_reservation.read();
            if show {
                let api_clone = api.clone();
                let mut reservations_clone = reservations.clone();
                let tables_data = tables.read().clone();
                let customers_data = customers.read().clone();
                rsx! {
                    div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
                        div { class: "bg-white rounded-3xl shadow-2xl max-w-md w-full mx-4 overflow-hidden",
                            div { class: "bg-gradient-to-r from-[#dc2381] to-[#b8186c] px-6 py-5",
                                div { class: "flex justify-between items-center",
                                    h3 { class: "text-xl font-bold text-white", "Add Reservation" }
                                    button {
                                        class: "w-10 h-10 bg-white/20 hover:bg-white/30 rounded-xl flex items-center justify-center text-white transition-all duration-200 backdrop-blur-sm",
                                        onclick: move |_| show_add_reservation.set(false),
                                        "×"
                                    }
                                }
                            }
                            div { class: "p-6 space-y-4",
                                div { class: "space-y-2",
                                    label { class: "block text-sm font-semibold text-gray-700", "Select Table" }
                                    select {
                                        class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#dc2381] focus:outline-none transition-colors",
                                        onchange: move |e| {
                                            if let Ok(uuid) = uuid::Uuid::parse_str(&e.value()) {
                                                selected_table.set(Some(uuid));
                                            }
                                        },
                                        option { value: "", "Select a table" }
                                        if let Some(tables) = &tables_data {
                                            for table in tables {
                                                option { value: "{table.id}", "Table {table.number} - {table.location.as_deref().unwrap_or(\"Unknown\")}" }
                                            }
                                        }
                                    }
                                }
                                div { class: "space-y-2",
                                    label { class: "block text-sm font-semibold text-gray-700", "Customer (optional)" }
                                    select {
                                        class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#dc2381] focus:outline-none transition-colors",
                                        onchange: move |e| {
                                            if let Ok(uuid) = uuid::Uuid::parse_str(&e.value()) {
                                                selected_customer.set(Some(uuid));
                                            }
                                        },
                                        option { value: "", "No customer" }
                                        if let Some(customers) = &customers_data {
                                            for customer in customers {
                                                option { value: "{customer.id}", "{customer.name}" }
                                            }
                                        }
                                    }
                                }
                                div { class: "space-y-2",
                                    label { class: "block text-sm font-semibold text-gray-700", "Reservation Time" }
                                    input {
                                        r#type: "datetime-local",
                                        class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#dc2381] focus:outline-none transition-colors",
                                        value: "{reservation_time}",
                                        oninput: move |e| reservation_time.set(e.value())
                                    }
                                }
                                div { class: "space-y-2",
                                    label { class: "block text-sm font-semibold text-gray-700", "Party Size" }
                                    input {
                                        r#type: "number",
                                        class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#dc2381] focus:outline-none transition-colors",
                                        value: "{party_size}",
                                        min: "1",
                                        oninput: move |e| {
                                            if let Ok(q) = e.value().parse::<i32>() {
                                                party_size.set(q.max(1));
                                            }
                                        }
                                    }
                                }
                                div { class: "space-y-2",
                                    label { class: "block text-sm font-semibold text-gray-700", "Notes (optional)" }
                                    textarea {
                                        class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#dc2381] focus:outline-none transition-colors resize-none",
                                        rows: 2,
                                        placeholder: "Special requests...",
                                        value: "{notes}",
                                        oninput: move |e| notes.set(e.value())
                                    }
                                }
                                div { class: "flex gap-3 pt-4",
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        onclick: move |_| show_add_reservation.set(false),
                                        children: rsx! { "Cancel" }
                                    }
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        onclick: move |_| {
                                            if let Some(table_id) = selected_table() {
                                                if let Ok(time) = chrono::DateTime::parse_from_rfc3339(&format!("{}:00Z", reservation_time())) {
                                                    let time = time.with_timezone(&chrono::Utc);
                                                    let customer_id = selected_customer();
                                                    let api_clone2 = api_clone.clone();
                                                    let mut reservations_clone2 = reservations_clone.clone();
                                                    dioxus::prelude::spawn(async move {
                                                        let _ = api_clone2.create_reservation(table_id, customer_id, time, party_size(), if notes().is_empty() { None } else { Some(notes()) }).await;
                                                        reservations_clone2.restart();
                                                    });
                                                    show_add_reservation.set(false);
                                                    selected_table.set(None);
                                                    selected_customer.set(None);
                                                    reservation_time.set(String::new());
                                                    party_size.set(2);
                                                    notes.set(String::new());
                                                }
                                            }
                                        },
                                        disabled: selected_table().is_none() || reservation_time().is_empty(),
                                        children: rsx! { "Add Reservation" }
                                    }
                                }
                            }
                        }
                    }
                }
            } else { rsx! {} }}
        }
    }
}

#[component]
fn ReservationCard(reservation: Reservation) -> Element {
    let status_variant = match reservation.status {
        kipko_core::ReservationStatus::Confirmed => BadgeVariant::Success,
        kipko_core::ReservationStatus::Seated => BadgeVariant::Primary,
        kipko_core::ReservationStatus::Cancelled => BadgeVariant::Danger,
        kipko_core::ReservationStatus::NoShow => BadgeVariant::Secondary,
    };
    let status_text = match reservation.status {
        kipko_core::ReservationStatus::Confirmed => "Confirmed",
        kipko_core::ReservationStatus::Seated => "Seated",
        kipko_core::ReservationStatus::Cancelled => "Cancelled",
        kipko_core::ReservationStatus::NoShow => "No Show",
    };
    let time_str = reservation.reservation_time.format("%Y-%m-%d %H:%M").to_string();

    rsx! {
        Card {
            class: "p-4",
            children: rsx! {
                div { class: "space-y-3",
                    div { class: "flex justify-between items-start",
                        h3 { class: "text-lg font-bold text-gray-900", "Table: {reservation.table_id}" }
                        Badge { variant: status_variant, children: rsx! { "{status_text}" } }
                    }
                    p { class: "text-sm text-gray-600", "📅 {time_str}" }
                    p { class: "text-sm text-gray-600", "👥 Party of {reservation.party_size}" }
                    if let Some(notes) = &reservation.notes {
                        p { class: "text-xs text-gray-500", "{notes}" }
                    }
                }
            }
        }
    }
}
