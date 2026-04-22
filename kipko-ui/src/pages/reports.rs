//! Reports Page

use dioxus::prelude::*;
use kipko_core::{Order, MenuItem};
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Card};

#[component]
pub fn ReportsPage() -> Element {
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
    let mut date_range = use_signal(|| "today".to_string());

    rsx! {
        div { class: "p-6 space-y-6",
            // Header
            div { class: "flex justify-between items-center",
                h1 { class: "text-3xl font-bold text-gray-900", "Reports" }
                Button {
                    variant: ButtonVariant::Outline,
                    onclick: move |_| {
                        orders.restart();
                        menu_items.restart();
                    },
                    children: rsx! { "Refresh" }
                }
            }

            // Date Range Selector
            div { class: "flex gap-3",
                Button {
                    variant: if date_range() == "today" { ButtonVariant::Primary } else { ButtonVariant::Outline },
                    onclick: move |_| date_range.set("today".to_string()),
                    children: rsx! { "Today" }
                }
                Button {
                    variant: if date_range() == "week" { ButtonVariant::Primary } else { ButtonVariant::Outline },
                    onclick: move |_| date_range.set("week".to_string()),
                    children: rsx! { "This Week" }
                }
                Button {
                    variant: if date_range() == "month" { ButtonVariant::Primary } else { ButtonVariant::Outline },
                    onclick: move |_| date_range.set("month".to_string()),
                    children: rsx! { "This Month" }
                }
            }

            // Reports Grid
            {let orders_data = orders.read();
            let menu_data = menu_items.read();
            if let Some(order_list) = orders_data.as_ref() {
                let total_revenue: rust_decimal::Decimal = order_list.iter()
                    .filter(|o| matches!(o.status, kipko_core::OrderStatus::Closed))
                    .map(|o| o.total_amount.amount())
                    .sum();
                let total_orders = order_list.iter()
                    .filter(|o| matches!(o.status, kipko_core::OrderStatus::Closed))
                    .count();
                let avg_order_value = if total_orders > 0 {
                    total_revenue / rust_decimal::Decimal::from(total_orders)
                } else {
                    rust_decimal::Decimal::ZERO
                };

                rsx! {
                    div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                        // Total Revenue
                        Card {
                            class: "p-6",
                            children: rsx! {
                                div { class: "space-y-2",
                                    p { class: "text-sm text-gray-600", "Total Revenue" }
                                    p { class: "text-3xl font-bold text-[#55aa86]", "KSH {total_revenue}" }
                                    p { class: "text-xs text-gray-500", "From {total_orders} closed orders" }
                                }
                            }
                        }

                        // Total Orders
                        Card {
                            class: "p-6",
                            children: rsx! {
                                div { class: "space-y-2",
                                    p { class: "text-sm text-gray-600", "Total Orders" }
                                    p { class: "text-3xl font-bold text-[#e0311f]", "{total_orders}" }
                                    p { class: "text-xs text-gray-500", "Closed orders" }
                                }
                            }
                        }

                        // Average Order Value
                        Card {
                            class: "p-6",
                            children: rsx! {
                                div { class: "space-y-2",
                                    p { class: "text-sm text-gray-600", "Average Order Value" }
                                    p { class: "text-3xl font-bold text-[#dc2381]", "KSH {avg_order_value}" }
                                    p { class: "text-xs text-gray-500", "Per order" }
                                }
                            }
                        }

                        // Popular Items
                        Card {
                            class: "p-6",
                            children: rsx! {
                                div { class: "space-y-3",
                                    p { class: "text-sm text-gray-600", "Popular Items" }
                                    {if let Some(items) = menu_data.as_ref() {
                                        let popular: Vec<_> = items.iter().take(5).collect();
                                        if popular.is_empty() {
                                            rsx! {
                                                p { class: "text-gray-500 italic", "No data" }
                                            }
                                        } else {
                                            rsx! {
                                                div { class: "space-y-2",
                                                    for item in popular {
                                                        div { class: "flex justify-between items-center",
                                                            p { class: "text-sm font-medium", "{item.name}" }
                                                            p { class: "text-sm text-gray-600", "KSH {item.price.amount()}" }
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

                        // Open Orders
                        Card {
                            class: "p-6",
                            children: rsx! {
                                div { class: "space-y-2",
                                    p { class: "text-sm text-gray-600", "Open Orders" }
                                    {let open_count = order_list.iter().filter(|o| matches!(o.status, kipko_core::OrderStatus::Open)).count();
                                    rsx! {
                                        p { class: "text-3xl font-bold text-[#55aa86]", "{open_count}" }
                                        p { class: "text-xs text-gray-500", "Currently active" }
                                    }}
                                }
                            }
                        }

                        // Cancelled Orders
                        Card {
                            class: "p-6",
                            children: rsx! {
                                div { class: "space-y-2",
                                    p { class: "text-sm text-gray-600", "Cancelled Orders" }
                                    {let cancelled_count = order_list.iter().filter(|o| matches!(o.status, kipko_core::OrderStatus::Cancelled)).count();
                                    rsx! {
                                        p { class: "text-3xl font-bold text-gray-600", "{cancelled_count}" }
                                        p { class: "text-xs text-gray-500", "Total cancelled" }
                                    }}
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
