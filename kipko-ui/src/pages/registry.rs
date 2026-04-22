//! Registry Page

use dioxus::prelude::*;
use kipko_core::RegistryEntry;
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Badge, BadgeVariant};

#[component]
pub fn RegistryPage() -> Element {
    let api = ApiService::new();
    let mut entries = use_resource(move || {
        let api_clone = api.clone();
        async move {
            api_clone.get_registry_entries().await.unwrap_or_default()
        }
    });

    let loading = entries.read().is_none();
    let entries_data = use_memo(move || entries.read().clone());

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                div { class: "space-y-1",
                    h2 { class: "text-3xl font-bold bg-gradient-to-r from-[#e0311f] to-[#dc2381] bg-clip-text text-transparent", "Registry" }
                    p { class: "text-gray-500", "System audit log and event history" }
                }
                Button {
                    variant: ButtonVariant::Secondary,
                    onclick: move |_| entries.restart(),
                    children: rsx! { "Refresh" }
                }
            }

            if loading {
                div { class: "flex flex-col items-center justify-center h-64 space-y-4",
                    div { class: "relative w-16 h-16",
                        div { class: "absolute inset-0 border-4 border-gray-200 rounded-full" }
                        div { class: "absolute inset-0 border-4 border-[#e0311f] rounded-full border-t-transparent animate-spin" }
                    }
                    p { class: "text-gray-500 font-medium", "Loading registry..." }
                }
            }

            {let data = entries_data();
            if let Some(entry_list) = data.as_ref() {
                if entry_list.is_empty() {
                    rsx! {
                        div { class: "text-center py-12 p-6 bg-white rounded-2xl shadow-xl shadow-gray-200/50 border border-gray-100",
                            p { class: "text-gray-500", "No registry entries found" }
                        }
                    }
                } else {
                    rsx! {
                        div { class: "bg-white rounded-2xl shadow-xl shadow-gray-200/50 border border-gray-100 overflow-hidden",
                            for entry in entry_list.iter() {
                                RegistryEntryRow { entry: entry.clone() }
                            }
                        }
                    }
                }
            } else { rsx! {} }}
        }
    }
}

#[component]
fn RegistryEntryRow(entry: RegistryEntry) -> Element {
    let action_variant = match entry.action.as_str() {
        "create" => BadgeVariant::Success,
        "update" => BadgeVariant::Warning,
        "delete" => BadgeVariant::Danger,
        _ => BadgeVariant::Secondary,
    };
    let created_at_str = entry.created_at.format("%Y-%m-%d %H:%M:%S").to_string();

    rsx! {
        div { class: "flex items-center justify-between p-4 border-b border-gray-100 last:border-0 hover:bg-gray-50 transition-colors",
            div { class: "flex items-center space-x-4",
                Badge { variant: action_variant, children: rsx! { "{entry.action}" } }
                div { class: "space-y-1",
                    p { class: "text-sm font-semibold text-gray-900", "{entry.entity_type}" }
                    p { class: "text-xs text-gray-500", "ID: {entry.entity_id}" }
                    if let Some(details) = &entry.details {
                        p { class: "text-xs text-gray-600", "{details}" }
                    }
                }
            }
            div { class: "text-right space-y-1",
                p { class: "text-xs text-gray-500", "User: {entry.created_by}" }
                p { class: "text-xs text-gray-500", "{created_at_str}" }
            }
        }
    }
}
