//! Form components
//! 
//! This module contains reusable form components.

use dioxus::prelude::*;

#[component]
pub fn InputField(
    label: String,
    name: String,
    value: String,
    on_input: EventHandler<FormEvent>,
    placeholder: Option<String>,
    input_type: Option<String>,
    error: Option<String>,
    required: Option<bool>,
) -> Element {
    let input_type = input_type.unwrap_or_else(|| "text".to_string());
    let placeholder = placeholder.unwrap_or_else(|| "".to_string());
    let required = required.unwrap_or(false);
    
    rsx! {
        div { class: "mb-4",
            label { 
                class: "block text-sm font-medium text-gray-700 mb-2",
                r#for: "{name}",
                "{label}"
                if required {
                    span { class: "text-red-500 ml-1", "*" }
                }
            }
            input {
                class: "w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-blue-500 focus:border-blue-500",
                r#type: "{input_type}",
                name: "{name}",
                id: "{name}",
                value: "{value}",
                placeholder: "{placeholder}",
                oninput: move |e| on_input(e),
                required: required
            }
            if let Some(err_msg) = error {
                p { class: "mt-1 text-sm text-red-600", "{err_msg}" }
            }
        }
    }
}

#[component]
pub fn Button(
    children: Element,
    onclick: EventHandler<MouseEvent>,
    variant: String,
    disabled: Option<bool>,
) -> Element {
    let base_classes = "px-4 py-2 rounded font-medium transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2";
    
    let variant_classes = match variant.as_str() {
        "primary" => "bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500",
        "secondary" => "bg-gray-600 text-white hover:bg-gray-700 focus:ring-gray-500",
        "success" => "bg-green-600 text-white hover:bg-green-700 focus:ring-green-500",
        "danger" => "bg-red-600 text-white hover:bg-red-700 focus:ring-red-500",
        _ => "bg-blue-600 text-white hover:bg-blue-700 focus:ring-blue-500",
    };
    
    let disabled_class = if disabled.unwrap_or(false) {
        "opacity-50 cursor-not-allowed"
    } else {
        ""
    };
    
    rsx! {
        button {
            class: "{base_classes} {variant_classes} {disabled_class}",
            onclick: move |e| onclick(e),
            disabled: disabled.unwrap_or(false),
            {children}
        }
    }
}
