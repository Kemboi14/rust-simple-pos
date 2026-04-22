//! Discounts Page

use dioxus::prelude::*;
use kipko_core::Discount;
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Card, Badge, BadgeVariant};

#[component]
pub fn DiscountsPage() -> Element {
    let api = ApiService::new();
    let mut show_add_discount = use_signal(|| false);
    let mut discount_code = use_signal(|| String::new());
    let mut discount_type = use_signal(|| "Percentage".to_string());
    let mut discount_value = use_signal(|| String::new());
    let mut min_order = use_signal(|| String::new());

    rsx! {
        div { class: "p-6 space-y-6",
            // Header
            div { class: "flex justify-between items-center",
                h1 { class: "text-3xl font-bold text-gray-900", "Discounts" }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| show_add_discount.set(true),
                    children: rsx! { "Add Discount" }
                }
            }

            // Placeholder for discounts list
            div { class: "text-center py-12",
                p { class: "text-gray-500 text-lg", "Discount management coming soon" }
                p { class: "text-sm text-gray-400 mt-2", "Backend discounts API needs to be implemented" }
            }

            // Add Discount Modal
            {let show = *show_add_discount.read();
            if show {
                rsx! {
                    div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
                        div { class: "bg-white rounded-3xl shadow-2xl max-w-md w-full mx-4 overflow-hidden",
                            div { class: "bg-gradient-to-r from-[#55aa86] to-[#3d8a6a] px-6 py-5",
                                div { class: "flex justify-between items-center",
                                    h3 { class: "text-xl font-bold text-white", "Add Discount" }
                                    button {
                                        class: "w-10 h-10 bg-white/20 hover:bg-white/30 rounded-xl flex items-center justify-center text-white transition-all duration-200 backdrop-blur-sm",
                                        onclick: move |_| show_add_discount.set(false),
                                        "×"
                                    }
                                }
                            }
                            div { class: "p-6 space-y-4",
                                div { class: "space-y-2",
                                    label { class: "block text-sm font-semibold text-gray-700", "Discount Code" }
                                    input {
                                        r#type: "text",
                                        class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#55aa86] focus:outline-none transition-colors",
                                        value: "{discount_code}",
                                        oninput: move |e| discount_code.set(e.value())
                                    }
                                }
                                div { class: "space-y-2",
                                    label { class: "block text-sm font-semibold text-gray-700", "Discount Type" }
                                    select {
                                        class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#55aa86] focus:outline-none transition-colors",
                                        onchange: move |e| discount_type.set(e.value()),
                                        option { value: "Percentage", "Percentage" }
                                        option { value: "FixedAmount", "Fixed Amount" }
                                    }
                                }
                                div { class: "space-y-2",
                                    label { class: "block text-sm font-semibold text-gray-700", "Value" }
                                    input {
                                        r#type: "number",
                                        class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#55aa86] focus:outline-none transition-colors",
                                        value: "{discount_value}",
                                        oninput: move |e| discount_value.set(e.value())
                                    }
                                }
                                div { class: "space-y-2",
                                    label { class: "block text-sm font-semibold text-gray-700", "Minimum Order (KSH)" }
                                    input {
                                        r#type: "number",
                                        class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#55aa86] focus:outline-none transition-colors",
                                        value: "{min_order}",
                                        oninput: move |e| min_order.set(e.value())
                                    }
                                }
                                div { class: "flex gap-3 pt-4",
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        onclick: move |_| show_add_discount.set(false),
                                        children: rsx! { "Cancel" }
                                    }
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        onclick: move |_| {
                                            show_add_discount.set(false);
                                            discount_code.set(String::new());
                                            discount_value.set(String::new());
                                            min_order.set(String::new());
                                        },
                                        disabled: discount_code().is_empty() || discount_value().is_empty(),
                                        children: rsx! { "Add Discount" }
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
