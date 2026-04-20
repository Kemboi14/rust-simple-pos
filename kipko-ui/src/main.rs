//! Kipko POS UI

use dioxus::prelude::*;

mod components;
mod pages;
mod services;
mod utils;


fn main() {
    // Launch the web app
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut current_page = use_signal(|| "floorplan".to_string());
    
    // Pre-compute navigation button classes
    let floorplan_class = use_memo(move || {
        if current_page() == "floorplan" {
            "px-3 py-2 rounded hover:bg-blue-700 transition-colors bg-blue-700"
        } else {
            "px-3 py-2 rounded hover:bg-blue-700 transition-colors"
        }
    });
    let orders_class = use_memo(move || {
        if current_page() == "orders" {
            "px-3 py-2 rounded hover:bg-blue-700 transition-colors bg-blue-700"
        } else {
            "px-3 py-2 rounded hover:bg-blue-700 transition-colors"
        }
    });
    let menu_class = use_memo(move || {
        if current_page() == "menu" {
            "px-3 py-2 rounded hover:bg-blue-700 transition-colors bg-blue-700"
        } else {
            "px-3 py-2 rounded hover:bg-blue-700 transition-colors"
        }
    });
    let staff_class = use_memo(move || {
        if current_page() == "staff" {
            "px-3 py-2 rounded hover:bg-blue-700 transition-colors bg-blue-700"
        } else {
            "px-3 py-2 rounded hover:bg-blue-700 transition-colors"
        }
    });
    
    let page_content = use_memo(move || {
        let page = current_page().clone();
        match page.as_str() {
            "floorplan" => rsx! { pages::FloorPlan {} },
            "orders" => rsx! { pages::Orders {} },
            "menu" => rsx! { pages::Menu {} },
            "staff" => rsx! { pages::Staff {} },
            _ => rsx! { p { "Page not found" } }
        }
    });
    
    rsx! {
        div { class: "min-h-screen bg-gray-100",
            // Header
            header { class: "bg-blue-600 text-white shadow-lg",
                div { class: "container mx-auto px-4 py-4",
                    div { class: "flex justify-between items-center",
                        h1 { class: "text-2xl font-bold", "Kipko POS" }
                        nav { class: "flex space-x-4",
                            button {
                                class: "{floorplan_class}",
                                onclick: move |_| current_page.set("floorplan".to_string()),
                                "Floor Plan"
                            }
                            button {
                                class: "{orders_class}",
                                onclick: move |_| current_page.set("orders".to_string()),
                                "Orders"
                            }
                            button {
                                class: "{menu_class}",
                                onclick: move |_| current_page.set("menu".to_string()),
                                "Menu"
                            }
                            button {
                                class: "{staff_class}",
                                onclick: move |_| current_page.set("staff".to_string()),
                                "Staff"
                            }
                        }
                    }
                }
            }
            
            // Main Content
            main { class: "container mx-auto px-4 py-8",
                {page_content()}
            }
        }
    }
}
