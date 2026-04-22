//! Login Page

use dioxus::prelude::*;
use crate::components::{Button, ButtonVariant};

#[component]
pub fn LoginPage() -> Element {
    let mut username = use_signal(|| String::new());
    let mut password = use_signal(|| String::new());
    let mut is_authenticated = use_signal(|| false);

    rsx! {
        div { class: "min-h-screen bg-gradient-to-br from-[#e0311f] to-[#dc2381] flex items-center justify-center p-4",
            div { class: "bg-white rounded-3xl shadow-2xl max-w-md w-full p-8 space-y-6",
                // Header
                div { class: "text-center space-y-2",
                    div { class: "w-16 h-16 bg-gradient-to-br from-[#e0311f] to-[#dc2381] rounded-2xl flex items-center justify-center mx-auto shadow-lg shadow-[#e0311f]/30",
                        span { class: "text-white font-bold text-3xl", "K" }
                    }
                    h1 { class: "text-2xl font-bold text-gray-900", "Kipko POS" }
                    p { class: "text-gray-600", "Sign in to your account" }
                }

                {if !is_authenticated() {
                    rsx! {
                        // Login Form
                        div { class: "space-y-4",
                            div { class: "space-y-2",
                                label { class: "block text-sm font-semibold text-gray-700", "Username" }
                                input {
                                    r#type: "text",
                                    class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                                    value: "{username}",
                                    oninput: move |e| username.set(e.value())
                                }
                            }
                            div { class: "space-y-2",
                                label { class: "block text-sm font-semibold text-gray-700", "Password" }
                                input {
                                    r#type: "password",
                                    class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                                    value: "{password}",
                                    oninput: move |e| password.set(e.value())
                                }
                            }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| {
                                    if !username().is_empty() && !password().is_empty() {
                                        is_authenticated.set(true);
                                    }
                                },
                                disabled: username().is_empty() || password().is_empty(),
                                class: "w-full",
                                children: rsx! { "Sign In" }
                            }
                        }
                    }
                } else {
                    rsx! {
                        // Success Message
                        div { class: "text-center space-y-4",
                            div { class: "w-16 h-16 bg-[#55aa86] rounded-full flex items-center justify-center mx-auto",
                                span { class: "text-white text-3xl", "✓" }
                            }
                            h2 { class: "text-xl font-bold text-gray-900", "Welcome!" }
                            p { class: "text-gray-600", "You are now logged in" }
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| is_authenticated.set(false),
                                children: rsx! { "Logout" }
                            }
                        }
                    }
                }}
            }
        }
    }
}
