//! Kipko POS UI

use dioxus::prelude::*;

mod components;
mod pages;
mod services;

use pages::{FloorPlan, Orders, Menu, Staff};

fn main() {
    // Launch the app with SSR
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let current_page = use_signal(|| "floorplan".to_string());
    
    rsx! {
        div { class: "min-h-screen bg-gray-100",
            // Header
            header { class: "bg-blue-600 text-white shadow-lg",
                div { class: "container mx-auto px-4 py-4",
                    div { class: "flex justify-between items-center",
                        h1 { class: "text-2xl font-bold", "Kipko POS" }
                        nav { class: "flex space-x-4",
                            button {
                                class: "px-3 py-2 rounded hover:bg-blue-700 {if current_page() == \"floorplan\" { \"bg-blue-700\" } else { \"\" }}",
                                onclick: move |_| current_page.set("floorplan".to_string()),
                                "Floor Plan"
                            }
                            button {
                                class: "px-3 py-2 rounded hover:bg-blue-700 {if current_page() == \"orders\" { \"bg-blue-700\" } else { \"\" }}",
                                onclick: move |_| current_page.set("orders".to_string()),
                                "Orders"
                            }
                            button {
                                class: "px-3 py-2 rounded hover:bg-blue-700 {if current_page() == \"menu\" { \"bg-blue-700\" } else { \"\" }}",
                                onclick: move |_| current_page.set("menu".to_string()),
                                "Menu"
                            }
                            button {
                                class: "px-3 py-2 rounded hover:bg-blue-700 {if current_page() == \"staff\" { \"bg-blue-700\" } else { \"\" }}",
                                onclick: move |_| current_page.set("staff".to_string()),
                                "Staff"
                            }
                        }
                    }
                }
            }
            
            // Main Content
            main { class: "container mx-auto px-4 py-8",
                div {
                    match current_page().as_str() {
                        "floorplan" => rsx! { pages::FloorPlan {} },
                        "orders" => rsx! { pages::Orders {} },
                        "menu" => rsx! { pages::Menu {} },
                        "staff" => rsx! { pages::Staff {} },
                        _ => rsx! { p { "Page not found" } }
                    }
                }
            }
        }
    }
}
