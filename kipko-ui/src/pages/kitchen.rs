//! Kitchen Display Page

use dioxus::prelude::*;
use kipko_core::{Order, OrderItem, MenuItem};
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Badge, BadgeVariant, Card};

#[component]
pub fn KitchenPage() -> Element {
    let api = ApiService::new();
    let api_clone1 = api.clone();
    let mut orders = use_resource(move || {
        let api_clone = api_clone1.clone();
        async move {
            api_clone.get_orders().await.unwrap_or_default()
        }
    });
    let api_clone2 = api.clone();
    let mut menu_items = use_resource(move || {
        let api_clone = api_clone2.clone();
        async move {
            api_clone.get_menu_items().await.unwrap_or_default()
        }
    });

    rsx! {
        div { class: "p-6 space-y-6",
            // Header
            div { class: "flex justify-between items-center",
                h1 { class: "text-3xl font-bold text-gray-900", "Kitchen Display" }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        orders.restart();
                        menu_items.restart();
                    },
                    children: rsx! { "Refresh" }
                }
            }

            // Kitchen Orders
            {let orders_data = orders.read();
            let menu_data = menu_items.read();
            if let Some(order_list) = orders_data.as_ref() {
                let open_orders: Vec<_> = order_list.iter().filter(|o| matches!(o.status, kipko_core::OrderStatus::Open)).collect();
                if open_orders.is_empty() {
                    rsx! {
                        div { class: "text-center py-12",
                            p { class: "text-gray-500 text-lg", "No active orders in kitchen" }
                        }
                    }
                } else {
                    rsx! {
                        div { class: "grid grid-cols-1 lg:grid-cols-2 gap-6",
                            for order in open_orders {
                                KitchenOrderCard {
                                    order: order.clone(),
                                    menu_items: menu_data.clone()
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
        }
    }
}

#[component]
fn KitchenOrderCard(order: Order, menu_items: Option<Vec<MenuItem>>) -> Element {
    let api = ApiService::new();
    let mut order_items = use_resource(move || {
        let api_clone = api.clone();
        async move {
            api_clone.get_order_items(order.id).await.unwrap_or_default()
        }
    });

    let order_id = order.id.to_string();
    let short_id = &order_id[..8.min(order_id.len())];

    rsx! {
        Card {
            class: "p-5",
            children: rsx! {
                div { class: "space-y-4",
                    // Order Header
                    div { class: "flex justify-between items-start border-b border-gray-200 pb-3",
                        div { class: "space-y-1",
                            h3 { class: "text-lg font-bold text-gray-900", "Order {short_id}" }
                            p { class: "text-sm text-gray-600", "Table: {order.table_id}" }
                        }
                        Badge { variant: BadgeVariant::Success, children: rsx! { "Active" } }
                    }

                    // Order Items
                    {let items = order_items.read();
                    if let Some(item_list) = items.as_ref() {
                        if item_list.is_empty() {
                            rsx! {
                                p { class: "text-gray-500 italic", "No items" }
                            }
                        } else {
                            rsx! {
                                div { class: "space-y-3",
                                    for item in item_list {
                                        KitchenItemRow {
                                            item: item.clone(),
                                            menu_items: menu_items.clone()
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "flex justify-center py-4",
                                div { class: "animate-spin rounded-full h-6 w-6 border-b-2 border-[#e0311f]" }
                            }
                        }
                    }}
                }
            }
        }
    }
}

#[component]
fn KitchenItemRow(item: OrderItem, menu_items: Option<Vec<MenuItem>>) -> Element {
    let api = ApiService::new();
    let mut item_status = use_signal(|| match item.status {
        kipko_core::OrderItemStatus::Pending => "Pending",
        kipko_core::OrderItemStatus::Fired => "Preparing",
        kipko_core::OrderItemStatus::Ready => "Ready",
        kipko_core::OrderItemStatus::Delivered => "Served",
        kipko_core::OrderItemStatus::Voided => "Voided",
    }.to_string());

    let status_variant = match item_status().as_str() {
        "Pending" => BadgeVariant::Secondary,
        "Preparing" => BadgeVariant::Primary,
        "Ready" => BadgeVariant::Success,
        "Served" => BadgeVariant::Secondary,
        "Voided" => BadgeVariant::Danger,
        _ => BadgeVariant::Secondary,
    };

    let menu_item_name = menu_items.as_ref()
        .and_then(|items| items.iter().find(|m| m.id == item.menu_item_id))
        .map(|m| m.name.clone())
        .unwrap_or_else(|| "Unknown Item".to_string());

    rsx! {
        div { class: "flex justify-between items-center p-3 bg-gray-50 rounded-xl",
            div { class: "space-y-1",
                p { class: "font-medium text-gray-900", "{menu_item_name}" }
                p { class: "text-sm text-gray-600", "Qty: {item.quantity}" }
                if let Some(notes) = &item.notes {
                    p { class: "text-xs text-gray-500", "{notes}" }
                }
            }
            div { class: "flex items-center gap-2",
                Badge { variant: status_variant, children: rsx! { "{item_status()}" } }
                if matches!(item.status, kipko_core::OrderItemStatus::Pending | kipko_core::OrderItemStatus::Fired) {
                    Button {
                        variant: ButtonVariant::Outline,
                        class: "text-xs px-2 py-1",
                        onclick: move |_| {
                            let new_status = if item_status() == "Pending" { "Preparing" } else { "Ready" };
                            item_status.set(new_status.to_string());
                            let api_clone = api.clone();
                            let order_id = item.order_id;
                            let item_id = item.id;
                            dioxus::prelude::spawn(async move {
                                let _ = api_clone.update_order_item(order_id, item_id, None, Some(new_status.to_string())).await;
                            });
                        },
                        children: rsx! { if item_status() == "Pending" { "Start" } else { "Mark Ready" } }
                    }
                }
            }
        }
    }
}
