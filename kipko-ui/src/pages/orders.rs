//! Orders Page

use dioxus::prelude::*;
use kipko_core::{Order, OrderStatus};
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Badge, BadgeVariant, QRCodeDisplay};

#[component]
pub fn Orders() -> Element {
    let api = ApiService::new();
    let mut orders = use_resource(move || {
        let api_clone = api.clone();
        async move {
            api_clone.get_orders().await.unwrap_or_default()
        }
    });
    
    let mut selected_order = use_signal(|| Option::<uuid::Uuid>::None);
    let mut show_details = use_signal(|| false);
    let loading = orders.read().is_none();
    
    let orders_data = use_memo(move || orders.read().clone());

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                h2 { class: "text-2xl font-bold text-gray-900", "Orders" }
                div { class: "flex gap-2",
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| {},
                        children: rsx! { "New Order" }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| { orders.restart(); },
                        children: rsx! { "Refresh" }
                    }
                }
            }

            if loading {
                div { class: "flex justify-center items-center h-64",
                    div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" }
                }
            }

            {let data = orders_data();
            if let Some(order_list) = data.as_ref() {
                if order_list.is_empty() {
                    rsx! {
                        div { class: "text-center py-12 p-4 bg-white rounded-lg shadow",
                            p { class: "text-gray-500", "No orders found" }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| {},
                                children: rsx! { "Create First Order" }
                            }
                        }
                    }
                } else {
                    rsx! {
                        div { class: "space-y-4",
                            for order in order_list {
                                {let sel = *selected_order.read() == Some(order.id);
                                let cls = if sel { "cursor-pointer hover:shadow-lg transition-shadow p-4 bg-white rounded-lg shadow ring-2 ring-blue-500" } else { "cursor-pointer hover:shadow-lg transition-shadow p-4 bg-white rounded-lg shadow" };
                                rsx! {
                                    div {
                                        class: "{cls}",
                                        onclick: move |_| {
                                            selected_order.set(Some(order.id));
                                            show_details.set(true);
                                        },
                                        OrderCardContent { order: order.clone() }
                                    }
                                }}
                            }
                        }
                    }
                }
            } else { rsx! {} }}

            {let show = *show_details.read();
            let sel_id = *selected_order.read();
            let data2 = orders_data();
            if show {
                if let Some(selected_id) = sel_id {
                    if let Some(order) = data2.as_ref().and_then(|l| l.iter().find(|o| o.id == selected_id)).cloned() {
                        rsx! {
                            OrderDetails {
                                order: order,
                                on_close: move |_| {
                                    show_details.set(false);
                                    selected_order.set(None);
                                },
                                on_close_order: move |_| {}
                            }
                        }
                    } else { rsx! {} }
                } else { rsx! {} }
            } else { rsx! {} }}
        }
    }
}

#[component]
fn OrderCardContent(order: Order) -> Element {
    let status_variant = match order.status {
        OrderStatus::Open => BadgeVariant::Success,
        OrderStatus::Closed => BadgeVariant::Secondary,
        OrderStatus::Cancelled => BadgeVariant::Danger,
    };
    let status_text = match order.status {
        OrderStatus::Open => "Open",
        OrderStatus::Closed => "Closed",
        OrderStatus::Cancelled => "Cancelled",
    };
    let order_id = order.id.to_string();
    let short_id = &order_id[..8.min(order_id.len())];
    let table_id = order.table_id.to_string();
    let short_table = &table_id[..8.min(table_id.len())];
    let staff_id = order.staff_id.to_string();
    let short_staff = &staff_id[..8.min(staff_id.len())];

    rsx! {
        div { class: "flex justify-between items-start",
            div { class: "space-y-2",
                div { class: "flex items-center gap-3",
                    h3 { class: "text-lg font-semibold text-gray-900", "Order #{short_id}" }
                    Badge { variant: status_variant, children: rsx! { "{status_text}" } }
                }
                p { class: "text-sm text-gray-600", "Table: {short_table} / Staff: {short_staff}" }
            }
        }
    }
}

#[component]
fn OrderDetails(
    order: Order,
    on_close: EventHandler<MouseEvent>,
    on_close_order: EventHandler<MouseEvent>,
) -> Element {
    let status_variant = match order.status {
        OrderStatus::Open => BadgeVariant::Success,
        OrderStatus::Closed => BadgeVariant::Secondary,
        OrderStatus::Cancelled => BadgeVariant::Danger,
    };
    let status_text = match order.status {
        OrderStatus::Open => "Open",
        OrderStatus::Closed => "Closed",
        OrderStatus::Cancelled => "Cancelled",
    };
    let mut show_qr = use_signal(|| false);
    let order_id = order.id.to_string();
    let short_id = &order_id[..8.min(order_id.len())];

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto bg-white rounded-lg shadow p-6",
                div { class: "flex justify-between items-start mb-4",
                    div {
                        h3 { class: "text-xl font-bold text-gray-900", "Order Details" }
                        p { class: "text-sm text-gray-500", "ID: {short_id}" }
                    }
                    button { class: "text-gray-400 hover:text-gray-600 text-2xl", onclick: move |e| on_close(e), "×" }
                }
                div { class: "space-y-6",
                    div { class: "flex items-center justify-between p-4 bg-gray-50 rounded-lg",
                        div { class: "space-y-1",
                            div { class: "flex items-center gap-2",
                                span { class: "text-gray-600", "Status:" }
                                Badge { variant: status_variant, children: rsx! { "{status_text}" } }
                            }
                        }
                    }
                    if matches!(order.status, OrderStatus::Open) {
                        div { class: "border-t pt-4 flex gap-2",
                            Button { variant: ButtonVariant::Success, onclick: move |e| on_close_order(e), children: rsx! { "Close Order" } }
                            Button { variant: ButtonVariant::Outline, onclick: move |_| {}, children: rsx! { "Add Items" } }
                        }
                    }
                }
            }
        }
    }
}
