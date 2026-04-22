//! Common UI components
//! 
//! This module contains reusable UI components used throughout the application.

use dioxus::prelude::*;

#[component]
pub fn LoadingSpinner() -> Element {
    rsx! {
        div { class: "flex justify-center items-center",
            div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600" }
        }
    }
}

#[component]
pub fn ErrorMessage(message: String) -> Element {
    rsx! {
        div { class: "bg-red-100 border border-red-400 text-red-700 px-4 py-3 rounded",
            strong { "Error: " }
            {message}
        }
    }
}

#[component]
pub fn SuccessMessage(message: String) -> Element {
    rsx! {
        div { class: "bg-green-100 border border-green-400 text-green-700 px-4 py-3 rounded",
            strong { "Success: " }
            {message}
        }
    }
}

#[component]
pub fn Button(
    children: Element,
    onclick: EventHandler<MouseEvent>,
    variant: ButtonVariant,
    disabled: Option<bool>,
    class: Option<String>,
) -> Element {
    let base_classes = "px-5 py-2.5 rounded-xl font-medium transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 transform hover:scale-105 active:scale-95";
    
    let variant_classes = match variant {
        ButtonVariant::Primary => "bg-gradient-to-r from-[#e0311f] to-[#dc2381] text-white hover:from-[#c4211a] hover:to-[#c41d70] focus:ring-[#e0311f] shadow-lg shadow-[#e0311f]/30",
        ButtonVariant::Secondary => "bg-gradient-to-r from-[#55aa86] to-[#4a9a76] text-white hover:from-[#4a9a76] hover:to-[#3f8a66] focus:ring-[#55aa86] shadow-lg shadow-[#55aa86]/30",
        ButtonVariant::Success => "bg-gradient-to-r from-[#55aa86] to-[#6bc29a] text-white hover:from-[#4a9a76] hover:to-[#5ab28a] focus:ring-[#55aa86] shadow-lg shadow-[#55aa86]/30",
        ButtonVariant::Danger => "bg-gradient-to-r from-[#e0311f] to-[#ff4757] text-white hover:from-[#c4211a] hover:to-[#e0383a] focus:ring-[#e0311f] shadow-lg shadow-[#e0311f]/30",
        ButtonVariant::Outline => "border-2 border-gray-300 text-gray-700 bg-white hover:bg-gray-50 hover:border-gray-400 focus:ring-gray-500",
    };
    
    let disabled_class = if disabled.unwrap_or(false) {
        "opacity-50 cursor-not-allowed transform-none"
    } else {
        ""
    };
    
    let additional_class = class.unwrap_or_default();
    
    rsx! {
        button {
            class: "{base_classes} {variant_classes} {disabled_class} {additional_class}",
            onclick: move |e| onclick(e),
            disabled: disabled.unwrap_or(false),
            {children}
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Success,
    Danger,
    Outline,
}

#[component]
pub fn Card(
    children: Element,
    class: Option<String>,
    padding: Option<CardPadding>,
) -> Element {
    let base_classes = "bg-white rounded-2xl shadow-xl shadow-slate-200/50 border border-slate-100";
    
    let padding_classes = match padding.unwrap_or(CardPadding::Medium) {
        CardPadding::None => "",
        CardPadding::Small => "p-5",
        CardPadding::Medium => "p-6",
        CardPadding::Large => "p-8",
    };
    
    let additional_class = class.unwrap_or_default();
    
    rsx! {
        div { class: "{base_classes} {padding_classes} {additional_class}",
            {children}
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum CardPadding {
    None,
    Small,
    Medium,
    Large,
}

#[component]
pub fn Modal(
    children: Element,
    open: bool,
    on_close: EventHandler<MouseEvent>,
    title: Option<String>,
) -> Element {
    rsx! {
        if open {
            div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
                div { class: "bg-white rounded-lg shadow-xl max-w-md w-full mx-4",
                    if let Some(t) = title {
                        div { class: "flex justify-between items-center p-6 border-b",
                            h3 { class: "text-lg font-semibold", "{t}" }
                            button {
                                class: "text-gray-400 hover:text-gray-600",
                                onclick: move |e| on_close(e),
                                "×"
                            }
                        }
                    }
                    div { class: "p-6",
                        {children}
                    }
                }
            }
        }
    }
}

#[component]
pub fn Badge(
    children: Element,
    variant: BadgeVariant,
    size: Option<BadgeSize>,
) -> Element {
    let base_classes = "inline-flex items-center font-semibold rounded-xl";
    
    let variant_classes = match variant {
        BadgeVariant::Primary => "bg-gradient-to-r from-[#e0311f]/10 to-[#dc2381]/10 text-[#e0311f] border border-[#e0311f]/30",
        BadgeVariant::Secondary => "bg-gradient-to-r from-gray-100 to-gray-200 text-gray-800 border border-gray-300",
        BadgeVariant::Success => "bg-gradient-to-r from-[#55aa86]/10 to-[#6bc29a]/10 text-[#55aa86] border border-[#55aa86]/30",
        BadgeVariant::Danger => "bg-gradient-to-r from-[#e0311f]/10 to-[#ff4757]/10 text-[#e0311f] border border-[#e0311f]/30",
        BadgeVariant::Warning => "bg-gradient-to-r from-amber-100 to-yellow-100 text-amber-800 border border-amber-200",
    };
    
    let size_classes = match size.unwrap_or(BadgeSize::Medium) {
        BadgeSize::Small => "px-2.5 py-1 text-xs",
        BadgeSize::Medium => "px-3 py-1 text-sm",
        BadgeSize::Large => "px-4 py-1.5 text-base",
    };
    
    rsx! {
        span { class: "{base_classes} {variant_classes} {size_classes}",
            {children}
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum BadgeVariant {
    Primary,
    Secondary,
    Success,
    Danger,
    Warning,
}

#[derive(Clone, PartialEq)]
pub enum BadgeSize {
    Small,
    Medium,
    Large,
}

#[component]
pub fn QRCodeDisplay(
    data: String,
    size: Option<u32>,
    title: Option<String>,
    description: Option<String>,
) -> Element {
    let size = size.unwrap_or(200);
    let qr_data = crate::utils::generate_qr_code(&data, size).unwrap_or_default();
    
    rsx! {
        div { class: "flex flex-col items-center space-y-4",
            if let Some(t) = title {
                h3 { class: "text-lg font-semibold text-gray-900", "{t}" }
            }
            div { class: "bg-white p-4 rounded-lg shadow-md",
                img {
                    src: "{qr_data}",
                    alt: "QR Code",
                    class: "w-{size}px h-{size}px",
                    width: "{size}",
                    height: "{size}"
                }
            }
            if let Some(desc) = description {
                p { class: "text-sm text-gray-600 text-center", "{desc}" }
            }
        }
    }
}
