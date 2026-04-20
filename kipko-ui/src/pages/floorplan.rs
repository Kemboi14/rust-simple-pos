//! Floor Plan Page

use dioxus::prelude::*;
use kipko_core::{Table, TableStatus};
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Badge, BadgeVariant, QRCodeDisplay};

#[component]
pub fn FloorPlan() -> Element {
    let api = ApiService::new();
    let mut tables = use_resource(move || {
        let api_clone = api.clone();
        async move {
            api_clone.get_tables().await.unwrap_or_default()
        }
    });
    
    let mut selected_table = use_signal(|| Option::<uuid::Uuid>::None);
    let mut show_details = use_signal(|| false);
    let loading = tables.read().is_none();
    
    // Cache the tables data for use in RSX
    let tables_data = use_memo(move || tables.read().clone());

    rsx! {
        div { class: "min-h-screen bg-gray-100",
            // Header
            div { class: "flex justify-between items-center",
                h2 { class: "text-2xl font-bold text-gray-900", "Floor Plan" }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| {
                        tables.restart();
                    },
                    children: rsx! { "Refresh" }
                }
            }

            // Loading state
            if loading {
                div { class: "flex justify-center items-center h-64",
                    div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" }
                }
            }

            // Table grid - render inline to avoid lifetime issues
            {let data = tables_data();
            if let Some(table_list) = data.as_ref() {
                let list = table_list.clone();
                let sel_table = *selected_table.read();
                rsx! {
                    div { class: "bg-white rounded-lg shadow-md p-6",
                        h3 { class: "text-lg font-semibold text-gray-900 mb-4", "All Tables" }
                        div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4",
                            for table in list.clone() {
                                TableComponent {
                                    table: table.clone(),
                                    selected: sel_table == Some(table.id),
                                    onclick: move |_| {
                                        selected_table.set(Some(table.id));
                                        show_details.set(true);
                                    }
                                }
                            }
                        }
                    }
                }
            } else { rsx! {} }}

            // Table details panel
            {let show = *show_details.read();
            let sel_id = *selected_table.read();
            let data2 = tables_data();
            if show {
                if let Some(selected_id) = sel_id {
                    if let Some(table) = data2.as_ref().and_then(|l| l.iter().find(|t| t.id == selected_id)).cloned() {
                        rsx! {
                            TableDetails {
                                table: table,
                                on_close: move |_| {
                                    show_details.set(false);
                                    selected_table.set(None);
                                },
                                on_action: move |action: String| {
                                    // Handle table actions
                                    match action.as_str() {
                                        "occupy" => {
                                            // Call API to occupy table
                                        }
                                        "clear" => {
                                            // Call API to clear table
                                        }
                                        "clean" => {
                                            // Call API to clean table
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    } else { rsx! {} }
                } else { rsx! {} }
            } else { rsx! {} }}
        }
    }
}

#[component]
fn TableComponent(table: Table, selected: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let status_color = match table.status {
        TableStatus::Empty => "bg-green-500 hover:bg-green-600",
        TableStatus::Occupied => "bg-yellow-500 hover:bg-yellow-600",
        TableStatus::Dirty => "bg-red-500 hover:bg-red-600",
        TableStatus::Reserved => "bg-gray-500 hover:bg-gray-600",
    };

    let selected_class = if selected { "ring-2 ring-blue-800 ring-offset-2 scale-105" } else { "" };

    rsx! {
        button {
            class: "w-24 h-24 rounded-lg text-white font-semibold text-sm transition-all transform hover:scale-105 {status_color} {selected_class}",
            onclick: move |e| onclick(e),
            div { class: "flex flex-col items-center justify-center h-full",
                span { class: "text-xl font-bold", "T{table.number}" }
                span { class: "text-xs opacity-90", "{table.capacity} seats" }
            }
        }
    }
}

#[component]
fn TableDetails(
    table: Table,
    on_close: EventHandler<MouseEvent>,
    on_action: EventHandler<String>,
) -> Element {
    let status_variant = match table.status {
        TableStatus::Empty => BadgeVariant::Success,
        TableStatus::Occupied => BadgeVariant::Warning,
        TableStatus::Dirty => BadgeVariant::Danger,
        TableStatus::Reserved => BadgeVariant::Secondary,
    };

    let status_text = match table.status {
        TableStatus::Empty => "Empty",
        TableStatus::Occupied => "Occupied",
        TableStatus::Dirty => "Dirty",
        TableStatus::Reserved => "Reserved",
    };

    let mut show_qr = use_signal(|| false);
    let base_url = "https://kipko.com"; // TODO: Get from environment

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "bg-white rounded-lg shadow-md max-w-md w-full mx-4 p-6",
                div { class: "flex justify-between items-start mb-4",
                    h3 { class: "text-xl font-bold text-gray-900", "Table {table.number}" }
                    button {
                        class: "text-gray-400 hover:text-gray-600 text-2xl",
                        onclick: move |e| on_close(e),
                        "×"
                    }
                }

                div { class: "space-y-2",
                    div { class: "flex items-center justify-between",
                        span { class: "text-gray-600", "Status:" }
                        Badge {
                            variant: status_variant,
                            children: rsx! { "{status_text}" }
                        }
                    }

                    div { class: "flex items-center justify-between",
                        span { class: "text-gray-600", "Capacity:" }
                        span { class: "font-semibold", "{table.capacity} seats" }
                    }

                    if let Some(location) = &table.location {
                        div { class: "flex items-center justify-between",
                            span { class: "text-gray-600", "Location:" }
                            span { class: "font-semibold", "{location}" }
                        }
                    }

                    // QR Code Section
                    div { class: "border-t pt-4 mt-4",
                        div { class: "flex justify-between items-center mb-3",
                            h4 { class: "font-semibold text-gray-900", "QR Code" }
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| show_qr.set(!show_qr()),
                                children: rsx! { if *show_qr.read() { "Hide" } else { "Show" } }
                            }
                        }
                        if *show_qr.read() {
                            QRCodeDisplay {
                                data: crate::utils::generate_table_qr_code(table.id, table.number, base_url),
                                size: Some(200),
                                title: Some("Scan to Order".to_string()),
                                description: Some("Customers can scan this QR code to view the menu and place orders".to_string()),
                            }
                        }
                    }

                    // Actions
                    div { class: "border-t pt-4 mt-4",
                        h4 { class: "font-semibold text-gray-900 mb-3", "Actions" }
                        div { class: "grid grid-cols-3 gap-2",
                            if matches!(table.status, TableStatus::Empty | TableStatus::Dirty) {
                                Button {
                                    variant: ButtonVariant::Primary,
                                    onclick: move |_| on_action("occupy".to_string()),
                                    children: rsx! { "Occupy" }
                                }
                            }
                            if matches!(table.status, TableStatus::Occupied) {
                                Button {
                                    variant: ButtonVariant::Outline,
                                    onclick: move |_| on_action("clear".to_string()),
                                    children: rsx! { "Clear" }
                                }
                            }
                            if matches!(table.status, TableStatus::Dirty) {
                                Button {
                                    variant: ButtonVariant::Success,
                                    onclick: move |_| on_action("clean".to_string()),
                                    children: rsx! { "Clean" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
