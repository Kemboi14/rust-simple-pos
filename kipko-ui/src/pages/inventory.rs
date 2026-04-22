//! Inventory Page

use dioxus::prelude::*;
use kipko_core::{MenuItem, InventoryTransaction, InventoryTransactionType};
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Badge, BadgeVariant};

#[component]
pub fn InventoryPage() -> Element {
    let api = ApiService::new();
    let api_clone1 = api.clone();
    let mut menu_items = use_resource(move || {
        let api_clone = api_clone1.clone();
        async move {
            api_clone.get_menu_items().await.unwrap_or_default()
        }
    });
    let api_clone2 = api.clone();
    let mut transactions = use_resource(move || {
        let api_clone = api_clone2.clone();
        async move {
            api_clone.get_inventory_transactions().await.unwrap_or_default()
        }
    });

    let loading = menu_items.read().is_none() || transactions.read().is_none();
    let menu_data = use_memo(move || menu_items.read().clone());
    let trans_data = use_memo(move || transactions.read().clone());

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                div { class: "space-y-1",
                    h2 { class: "text-3xl font-bold bg-gradient-to-r from-[#e0311f] to-[#dc2381] bg-clip-text text-transparent", "Inventory" }
                    p { class: "text-gray-500", "Manage stock and inventory transactions" }
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| {
                        menu_items.restart();
                        transactions.restart();
                    },
                    children: rsx! { "Refresh" }
                }
            }

            if loading {
                div { class: "flex flex-col items-center justify-center h-64 space-y-4",
                    div { class: "relative w-16 h-16",
                        div { class: "absolute inset-0 border-4 border-gray-200 rounded-full" }
                        div { class: "absolute inset-0 border-4 border-[#e0311f] rounded-full border-t-transparent animate-spin" }
                    }
                    p { class: "text-gray-500 font-medium", "Loading inventory..." }
                }
            }

            // Stock Overview
            {let items = menu_data();
            if let Some(item_list) = items.as_ref() {
                let low_stock = item_list.iter().filter(|i| i.stock_quantity <= i.low_stock_threshold && i.stock_quantity > 0).count();
                let out_of_stock = item_list.iter().filter(|i| i.stock_quantity == 0).count();
                let total_items = item_list.len();

                rsx! {
                    div { class: "grid grid-cols-1 md:grid-cols-3 gap-6",
                        div { class: "p-6 bg-white rounded-2xl shadow-xl shadow-gray-200/50 border border-gray-100",
                            div { class: "flex items-center justify-between",
                                div { class: "space-y-1",
                                    p { class: "text-sm text-gray-600", "Total Items" }
                                    p { class: "text-3xl font-bold text-gray-900", "{total_items}" }
                                }
                                div { class: "w-12 h-12 bg-[#55aa86]/10 rounded-xl flex items-center justify-center",
                                    span { class: "text-2xl", "📦" }
                                }
                            }
                        }
                        div { class: "p-6 bg-white rounded-2xl shadow-xl shadow-gray-200/50 border border-gray-100",
                            div { class: "flex items-center justify-between",
                                div { class: "space-y-1",
                                    p { class: "text-sm text-gray-600", "Low Stock" }
                                    p { class: "text-3xl font-bold text-amber-600", "{low_stock}" }
                                }
                                div { class: "w-12 h-12 bg-amber-100 rounded-xl flex items-center justify-center",
                                    span { class: "text-2xl", "⚠️" }
                                }
                            }
                        }
                        div { class: "p-6 bg-white rounded-2xl shadow-xl shadow-gray-200/50 border border-gray-100",
                            div { class: "flex items-center justify-between",
                                div { class: "space-y-1",
                                    p { class: "text-sm text-gray-600", "Out of Stock" }
                                    p { class: "text-3xl font-bold text-[#e0311f]", "{out_of_stock}" }
                                }
                                div { class: "w-12 h-12 bg-[#e0311f]/10 rounded-xl flex items-center justify-center",
                                    span { class: "text-2xl", "🚫" }
                                }
                            }
                        }
                    }
                }
            } else { rsx! {} }}

            // Items with Stock
            {let items = menu_data();
            if let Some(item_list) = items.as_ref() {
                if item_list.is_empty() {
                    rsx! {
                        div { class: "text-center py-12 p-6 bg-white rounded-2xl shadow-xl shadow-gray-200/50 border border-gray-100",
                            p { class: "text-gray-500", "No items found" }
                        }
                    }
                } else {
                    rsx! {
                        div { class: "space-y-4",
                            h3 { class: "text-xl font-bold text-gray-900", "Items by Stock Level" }
                            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                                for item in item_list.clone() {
                                    InventoryItemCard { item: item }
                                }
                            }
                        }
                    }
                }
            } else { rsx! {} }}

            // Recent Transactions
            {let trans = trans_data();
            if let Some(trans_list) = trans.as_ref() {
                if !trans_list.is_empty() {
                    rsx! {
                        div { class: "space-y-4",
                            h3 { class: "text-xl font-bold text-gray-900", "Recent Transactions" }
                            div { class: "bg-white rounded-2xl shadow-xl shadow-gray-200/50 border border-gray-100 overflow-hidden",
                                for transaction in trans_list.iter().take(10) {
                                    TransactionRow { transaction: transaction.clone() }
                                }
                            }
                        }
                    }
                } else { rsx! {} }
            } else { rsx! {} }}
        }
    }
}

#[component]
fn InventoryItemCard(item: MenuItem) -> Element {
    let stock_status = if item.stock_quantity == 0 {
        ("Out of Stock", "bg-[#e0311f]/10 text-[#e0311f] border-[#e0311f]/30")
    } else if item.stock_quantity <= item.low_stock_threshold {
        ("Low Stock", "bg-amber-100 text-amber-800 border-amber-200")
    } else {
        ("In Stock", "bg-[#55aa86]/10 text-[#55aa86] border-[#55aa86]/30")
    };

    rsx! {
        div { class: "p-5 bg-white rounded-2xl shadow-xl shadow-gray-200/50 border border-gray-100",
            div { class: "flex justify-between items-start mb-3",
                h4 { class: "text-lg font-bold text-gray-900", "{item.name}" }
                Badge { variant: BadgeVariant::Secondary, children: rsx! { "{stock_status.0}" } }
            }
            div { class: "flex items-center justify-between p-3 bg-gradient-to-r from-gray-50 to-gray-100 rounded-xl",
                div { class: "space-y-1",
                    p { class: "text-sm text-gray-600", "Current Stock" }
                    p { class: "text-2xl font-bold text-gray-900", "{item.stock_quantity}" }
                }
                div { class: "text-right space-y-1",
                    p { class: "text-sm text-gray-600", "Threshold" }
                    p { class: "text-lg font-semibold text-gray-700", "{item.low_stock_threshold}" }
                }
            }
        }
    }
}

#[component]
fn TransactionRow(transaction: InventoryTransaction) -> Element {
    let type_variant = match transaction.transaction_type {
        InventoryTransactionType::StockIn => BadgeVariant::Success,
        InventoryTransactionType::StockOut => BadgeVariant::Danger,
        InventoryTransactionType::Adjustment => BadgeVariant::Warning,
        InventoryTransactionType::Transfer => BadgeVariant::Secondary,
    };
    let type_text = match transaction.transaction_type {
        InventoryTransactionType::StockIn => "Stock In",
        InventoryTransactionType::StockOut => "Stock Out",
        InventoryTransactionType::Adjustment => "Adjustment",
        InventoryTransactionType::Transfer => "Transfer",
    };
    let created_at_str = transaction.created_at.format("%Y-%m-%d %H:%M").to_string();

    rsx! {
        div { class: "flex items-center justify-between p-4 border-b border-gray-100 last:border-0",
            div { class: "flex items-center space-x-4",
                Badge { variant: type_variant, children: rsx! { "{type_text}" } }
                div { class: "space-y-1",
                    p { class: "text-sm font-semibold text-gray-900", "Item ID: {transaction.menu_item_id}" }
                    if let Some(notes) = &transaction.notes {
                        p { class: "text-xs text-gray-500", "{notes}" }
                    }
                }
            }
            div { class: "text-right space-y-1",
                p { class: "text-lg font-bold text-gray-900", "{transaction.quantity}" }
                p { class: "text-xs text-gray-500", "{created_at_str}" }
            }
        }
    }
}
