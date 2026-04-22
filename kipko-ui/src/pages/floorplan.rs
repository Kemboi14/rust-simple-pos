//! Floor Plan Page

use dioxus::prelude::*;
use kipko_core::{Table, TableStatus};
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Badge, BadgeVariant, QRCodeDisplay};

#[component]
pub fn FloorPlan() -> Element {
    let api = ApiService::new();
    let api_for_resource = api.clone();
    let mut tables = use_resource(move || {
        let api_clone = api_for_resource.clone();
        async move {
            api_clone.get_tables().await.unwrap_or_default()
        }
    });
    
    let mut selected_table = use_signal(|| Option::<uuid::Uuid>::None);
    let mut show_details = use_signal(|| false);
    let mut show_add_table = use_signal(|| false);
    let loading = tables.read().is_none();
    
    // Cache the tables data for use in RSX
    let tables_data = use_memo(move || tables.read().clone());

    rsx! {
        div { class: "space-y-6",
            // Header with modern styling
            div { class: "flex justify-between items-center",
                div { class: "space-y-1",
                    h2 { class: "text-3xl font-bold bg-gradient-to-r from-[#e0311f] to-[#dc2381] bg-clip-text text-transparent", "Floor Plan" }
                    p { class: "text-gray-500", "Manage your restaurant tables" }
                }
                div { class: "flex gap-3",
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_add_table.set(true),
                        children: rsx! { "Add Table" }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            tables.restart();
                        },
                        children: rsx! { "Refresh" }
                    }
                }
            }

            // Loading state with modern spinner
            if loading {
                div { class: "flex flex-col items-center justify-center h-64 space-y-4",
                    div { class: "relative w-16 h-16",
                        div { class: "absolute inset-0 border-4 border-gray-200 rounded-full" }
                        div { class: "absolute inset-0 border-4 border-[#e0311f] rounded-full border-t-transparent animate-spin" }
                    }
                    p { class: "text-gray-500 font-medium", "Loading tables..." }
                }
            }

            // Table grid - render inline to avoid lifetime issues
            {let data = tables_data();
            if let Some(table_list) = data.as_ref() {
                let list = table_list.clone();
                let sel_table = *selected_table.read();
                rsx! {
                    div { class: "bg-white rounded-2xl shadow-xl shadow-slate-200/50 p-8 border border-slate-100",
                        div { class: "flex items-center justify-between mb-6",
                            h3 { class: "text-xl font-bold text-gray-900", "All Tables" }
                            div { class: "flex items-center space-x-2 text-sm text-gray-500",
                                span { class: "w-2 h-2 bg-green-500 rounded-full animate-pulse" }
                                span { "{list.len()} tables" }
                            }
                        }
                        div { class: "grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-6",
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
                        let table_id = table.id;
                        let api_clone = api.clone();
                        let tables_clone = tables.clone();
                        rsx! {
                            TableDetails {
                                table: table,
                                on_close: move |_| {
                                    show_details.set(false);
                                    selected_table.set(None);
                                },
                                on_action: move |action: String| {
                                    let api_clone2 = api_clone.clone();
                                    let table_id2 = table_id;
                                    let mut tables_clone2 = tables_clone.clone();
                                    dioxus::prelude::spawn(async move {
                                        match action.as_str() {
                                            "occupy" => {
                                                let _ = api_clone2.occupy_table(table_id2).await;
                                            }
                                            "clear" => {
                                                let _ = api_clone2.clear_table(table_id2).await;
                                            }
                                            "clean" => {
                                                let _ = api_clone2.clean_table(table_id2).await;
                                            }
                                            _ => {}
                                        }
                                        // Refresh tables after action
                                        tables_clone2.restart();
                                    });
                                }
                            }
                        }
                    } else { rsx! {} }
                } else { rsx! {} }
            } else { rsx! {} }}

            // Add Table Modal
            {let show = *show_add_table.read();
            if show {
                let api_clone = api.clone();
                let tables_clone = tables.clone();
                rsx! {
                    AddTableModal {
                        on_close: move |_| show_add_table.set(false),
                        on_submit: move |table_data: CreateTableData| {
                            let api_clone2 = api_clone.clone();
                            let mut tables_clone2 = tables_clone.clone();
                            dioxus::prelude::spawn(async move {
                                let _ = api_clone2.create_table(table_data).await;
                                tables_clone2.restart();
                            });
                            show_add_table.set(false);
                        }
                    }
                }
            } else { rsx! {} }}
        }
    }
}

#[derive(Clone)]
pub struct CreateTableData {
    pub number: i32,
    pub capacity: i32,
    pub location: String,
}

#[component]
fn AddTableModal(on_close: EventHandler<MouseEvent>, on_submit: EventHandler<CreateTableData>) -> Element {
    let mut number = use_signal(|| 1);
    let mut capacity = use_signal(|| 4);
    let mut location = use_signal(|| "Main Floor".to_string());

    rsx! {
        div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
            div { class: "bg-white rounded-3xl shadow-2xl max-w-md w-full mx-4 overflow-hidden",
                // Header
                div { class: "bg-gradient-to-r from-[#e0311f] to-[#dc2381] px-6 py-5",
                    div { class: "flex justify-between items-center",
                        h3 { class: "text-xl font-bold text-white", "Add New Table" }
                        button {
                            class: "w-10 h-10 bg-white/20 hover:bg-white/30 rounded-xl flex items-center justify-center text-white transition-all duration-200 backdrop-blur-sm",
                            onclick: move |e| on_close(e),
                            "×"
                        }
                    }
                }

                div { class: "p-6 space-y-5",
                    // Table Number
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Table Number" }
                        input {
                            r#type: "number",
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            value: "{number}",
                            oninput: move |e| {
                                if let Ok(n) = e.value().parse() {
                                    number.set(n);
                                }
                            }
                        }
                    }

                    // Capacity
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Capacity (seats)" }
                        input {
                            r#type: "number",
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            value: "{capacity}",
                            oninput: move |e| {
                                if let Ok(n) = e.value().parse() {
                                    capacity.set(n);
                                }
                            }
                        }
                    }

                    // Location
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Location" }
                        input {
                            r#type: "text",
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            value: "{location}",
                            oninput: move |e| location.set(e.value())
                        }
                    }

                    // Actions
                    div { class: "flex gap-3 pt-4",
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |e| on_close(e),
                            children: rsx! { "Cancel" }
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                on_submit(CreateTableData {
                                    number: number(),
                                    capacity: capacity(),
                                    location: location(),
                                });
                            },
                            children: rsx! { "Add Table" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn TableComponent(table: Table, selected: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let status_color = match table.status {
        TableStatus::Empty => "bg-gradient-to-br from-[#55aa86] to-[#6bc29a] hover:from-[#4a9a76] hover:to-[#5ab28a] shadow-[#55aa86]/30",
        TableStatus::Occupied => "bg-gradient-to-br from-[#e0311f] to-[#dc2381] hover:from-[#c4211a] hover:to-[#c41d70] shadow-[#e0311f]/30",
        TableStatus::Dirty => "bg-gradient-to-br from-gray-400 to-gray-500 hover:from-gray-500 hover:to-gray-600 shadow-gray-500/30",
        TableStatus::Reserved => "bg-gradient-to-br from-[#dc2381] to-[#e0311f] hover:from-[#c41d70] hover:to-[#c4211a] shadow-[#dc2381]/30",
    };

    let selected_class = if selected { "ring-4 ring-[#e0311f] ring-offset-4 scale-105 shadow-2xl" } else { "shadow-lg" };

    let status_icon = match table.status {
        TableStatus::Empty => "✓",
        TableStatus::Occupied => "👥",
        TableStatus::Dirty => "🧹",
        TableStatus::Reserved => "📅",
    };

    rsx! {
        button {
            class: "w-full aspect-square rounded-2xl text-white font-semibold transition-all duration-300 transform hover:scale-105 hover:-translate-y-1 {status_color} {selected_class} shadow-lg",
            onclick: move |e| onclick(e),
            div { class: "flex flex-col items-center justify-center h-full space-y-2",
                span { class: "text-3xl", "{status_icon}" }
                span { class: "text-2xl font-bold", "T{table.number}" }
                span { class: "text-xs opacity-90 font-medium", "{table.capacity} seats" }
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
        div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
            div { class: "bg-white rounded-3xl shadow-2xl max-w-lg w-full mx-4 overflow-hidden",
                // Header with gradient
                div { class: "bg-gradient-to-r from-indigo-500 to-purple-600 px-6 py-5",
                    div { class: "flex justify-between items-center",
                        div { class: "flex items-center space-x-3",
                            div { class: "w-12 h-12 bg-white/20 rounded-xl flex items-center justify-center backdrop-blur-sm",
                                span { class: "text-2xl font-bold text-white", "T{table.number}" }
                            }
                            div { class: "space-y-1",
                                h3 { class: "text-xl font-bold text-white", "Table Details" }
                                p { class: "text-white/80 text-sm", "{table.location.as_deref().unwrap_or(&\"Main Floor\".to_string())}" }
                            }
                        }
                        button {
                            class: "w-10 h-10 bg-white/20 hover:bg-white/30 rounded-xl flex items-center justify-center text-white transition-all duration-200 backdrop-blur-sm",
                            onclick: move |e| on_close(e),
                            "×"
                        }
                    }
                }

                div { class: "p-6 space-y-6",
                    // Status Badge
                    div { class: "flex items-center justify-between p-4 bg-gradient-to-r from-slate-50 to-slate-100 rounded-2xl",
                        span { class: "text-gray-600 font-medium", "Current Status" }
                        Badge {
                            variant: status_variant,
                            children: rsx! { "{status_text}" }
                        }
                    }

                    // Info Grid
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "p-4 bg-gradient-to-br from-indigo-50 to-purple-50 rounded-2xl",
                            div { class: "text-3xl mb-1", "👥" }
                            div { class: "text-sm text-gray-600", "Capacity" }
                            div { class: "text-xl font-bold text-gray-900", "{table.capacity} seats" }
                        }
                        div { class: "p-4 bg-gradient-to-br from-emerald-50 to-teal-50 rounded-2xl",
                            div { class: "text-3xl mb-1", "📍" }
                            div { class: "text-sm text-gray-600", "Location" }
                            div { class: "text-xl font-bold text-gray-900", "{table.location.as_deref().unwrap_or(&\"Main Floor\".to_string())}" }
                        }
                    }

                    // QR Code Section
                    div { class: "border-t border-slate-200 pt-6",
                        div { class: "flex justify-between items-center mb-4",
                            h4 { class: "font-bold text-gray-900 flex items-center space-x-2",
                                span { class: "text-xl", "📱" }
                                span { "QR Code" }
                            }
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
                    div { class: "border-t border-slate-200 pt-6",
                        h4 { class: "font-bold text-gray-900 mb-4 flex items-center space-x-2",
                            span { class: "text-xl", "⚡" }
                            span { "Quick Actions" }
                        }
                        div { class: "grid grid-cols-3 gap-3",
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
