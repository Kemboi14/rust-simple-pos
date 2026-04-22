//! Customers Page

use dioxus::prelude::*;
use kipko_core::Customer;
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Card};

#[component]
pub fn CustomersPage() -> Element {
    let api = ApiService::new();
    let api_clone1 = api.clone();
    let mut customers = use_resource(move || {
        let api_clone = api_clone1.clone();
        async move {
            api_clone.get_customers().await.unwrap_or_default()
        }
    });
    let mut show_add_customer = use_signal(|| false);
    let mut new_customer_name = use_signal(|| String::new());
    let mut new_customer_phone = use_signal(|| String::new());
    let mut new_customer_email = use_signal(|| String::new());

    rsx! {
        div { class: "p-6 space-y-6",
            // Header
            div { class: "flex justify-between items-center",
                h1 { class: "text-3xl font-bold text-gray-900", "Customers" }
                Button {
                    variant: ButtonVariant::Primary,
                    onclick: move |_| show_add_customer.set(true),
                    children: rsx! { "Add Customer" }
                }
            }

            // Refresh Button
            Button {
                variant: ButtonVariant::Outline,
                onclick: move |_| customers.restart(),
                children: rsx! { "Refresh" }
            }

            // Customers List
            {let data = customers.read();
            if let Some(customer_list) = data.as_ref() {
                if customer_list.is_empty() {
                    rsx! {
                        div { class: "text-center py-12",
                            p { class: "text-gray-500 text-lg", "No customers yet" }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| show_add_customer.set(true),
                                children: rsx! { "Add First Customer" }
                            }
                        }
                    }
                } else {
                    rsx! {
                        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                            for customer in customer_list {
                                CustomerCard {
                                    customer: customer.clone()
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

            // Add Customer Modal
            {let show = *show_add_customer.read();
            if show {
                let api_clone = api.clone();
                let mut customers_clone = customers.clone();
                rsx! {
                    div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
                        div { class: "bg-white rounded-3xl shadow-2xl max-w-md w-full mx-4 overflow-hidden",
                            div { class: "bg-gradient-to-r from-[#55aa86] to-[#3d8a6a] px-6 py-5",
                                div { class: "flex justify-between items-center",
                                    h3 { class: "text-xl font-bold text-white", "Add Customer" }
                                    button {
                                        class: "w-10 h-10 bg-white/20 hover:bg-white/30 rounded-xl flex items-center justify-center text-white transition-all duration-200 backdrop-blur-sm",
                                        onclick: move |_| show_add_customer.set(false),
                                        "×"
                                    }
                                }
                            }
                            div { class: "p-6 space-y-4",
                                div { class: "space-y-2",
                                    label { class: "block text-sm font-semibold text-gray-700", "Name" }
                                    input {
                                        r#type: "text",
                                        class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#55aa86] focus:outline-none transition-colors",
                                        value: "{new_customer_name}",
                                        oninput: move |e| new_customer_name.set(e.value())
                                    }
                                }
                                div { class: "space-y-2",
                                    label { class: "block text-sm font-semibold text-gray-700", "Phone (optional)" }
                                    input {
                                        r#type: "text",
                                        class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#55aa86] focus:outline-none transition-colors",
                                        value: "{new_customer_phone}",
                                        oninput: move |e| new_customer_phone.set(e.value())
                                    }
                                }
                                div { class: "space-y-2",
                                    label { class: "block text-sm font-semibold text-gray-700", "Email (optional)" }
                                    input {
                                        r#type: "email",
                                        class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#55aa86] focus:outline-none transition-colors",
                                        value: "{new_customer_email}",
                                        oninput: move |e| new_customer_email.set(e.value())
                                    }
                                }
                                div { class: "flex gap-3 pt-4",
                                    Button {
                                        variant: ButtonVariant::Outline,
                                        onclick: move |_| show_add_customer.set(false),
                                        children: rsx! { "Cancel" }
                                    }
                                    Button {
                                        variant: ButtonVariant::Primary,
                                        onclick: move |_| {
                                            let name = new_customer_name();
                                            let phone = if new_customer_phone().is_empty() { None } else { Some(new_customer_phone()) };
                                            let email = if new_customer_email().is_empty() { None } else { Some(new_customer_email()) };
                                            let api_clone2 = api_clone.clone();
                                            let mut customers_clone2 = customers_clone.clone();
                                            dioxus::prelude::spawn(async move {
                                                let _ = api_clone2.create_customer(name, phone, email).await;
                                                customers_clone2.restart();
                                            });
                                            show_add_customer.set(false);
                                            new_customer_name.set(String::new());
                                            new_customer_phone.set(String::new());
                                            new_customer_email.set(String::new());
                                        },
                                        disabled: new_customer_name().is_empty(),
                                        children: rsx! { "Add Customer" }
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
fn CustomerCard(customer: Customer) -> Element {
    let created_at_str = customer.created_at.format("%Y-%m-%d").to_string();

    rsx! {
        Card {
            class: "p-4",
            children: rsx! {
                div { class: "space-y-3",
                    div { class: "flex justify-between items-start",
                        h3 { class: "text-lg font-bold text-gray-900", "{customer.name}" }
                        if customer.loyalty_points > 0 {
                            span { class: "bg-[#dc2381] text-white text-xs px-2 py-1 rounded-full", "{customer.loyalty_points} pts" }
                        }
                    }
                    if let Some(phone) = &customer.phone {
                        p { class: "text-sm text-gray-600", "📞 {phone}" }
                    }
                    if let Some(email) = &customer.email {
                        p { class: "text-sm text-gray-600", "✉️ {email}" }
                    }
                    p { class: "text-xs text-gray-500", "Since {created_at_str}" }
                }
            }
        }
    }
}
