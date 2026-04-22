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
    
    // Pre-compute navigation button classes with modern styling
    let floorplan_class = use_memo(move || {
        if current_page() == "floorplan" {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 bg-[#e0311f] text-white shadow-lg shadow-[#e0311f]/30"
        } else {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 text-gray-600 hover:bg-gray-100 hover:text-gray-900"
        }
    });
    let orders_class = use_memo(move || {
        if current_page() == "orders" {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 bg-[#55aa86] text-white shadow-lg shadow-[#55aa86]/30"
        } else {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 text-gray-600 hover:bg-gray-100 hover:text-gray-900"
        }
    });
    let menu_class = use_memo(move || {
        if current_page() == "menu" {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 bg-[#dc2381] text-white shadow-lg shadow-[#dc2381]/30"
        } else {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 text-gray-600 hover:bg-gray-100 hover:text-gray-900"
        }
    });
    let staff_class = use_memo(move || {
        if current_page() == "staff" {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 bg-[#e0311f] text-white shadow-lg shadow-[#e0311f]/30"
        } else {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 text-gray-600 hover:bg-gray-100 hover:text-gray-900"
        }
    });
    let inventory_class = use_memo(move || {
        if current_page() == "inventory" {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 bg-[#55aa86] text-white shadow-lg shadow-[#55aa86]/30"
        } else {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 text-gray-600 hover:bg-gray-100 hover:text-gray-900"
        }
    });
    let registry_class = use_memo(move || {
        if current_page() == "registry" {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 bg-[#dc2381] text-white shadow-lg shadow-[#dc2381]/30"
        } else {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 text-gray-600 hover:bg-gray-100 hover:text-gray-900"
        }
    });
    let customers_class = use_memo(move || {
        if current_page() == "customers" {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 bg-[#55aa86] text-white shadow-lg shadow-[#55aa86]/30"
        } else {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 text-gray-600 hover:bg-gray-100 hover:text-gray-900"
        }
    });
    let reservations_class = use_memo(move || {
        if current_page() == "reservations" {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 bg-[#e0311f] text-white shadow-lg shadow-[#e0311f]/30"
        } else {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 text-gray-600 hover:bg-gray-100 hover:text-gray-900"
        }
    });
    let kitchen_class = use_memo(move || {
        if current_page() == "kitchen" {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 bg-[#55aa86] text-white shadow-lg shadow-[#55aa86]/30"
        } else {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 text-gray-600 hover:bg-gray-100 hover:text-gray-900"
        }
    });
    let reports_class = use_memo(move || {
        if current_page() == "reports" {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 bg-[#dc2381] text-white shadow-lg shadow-[#dc2381]/30"
        } else {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 text-gray-600 hover:bg-gray-100 hover:text-gray-900"
        }
    });
    let login_class = use_memo(move || {
        if current_page() == "login" {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 bg-[#55aa86] text-white shadow-lg shadow-[#55aa86]/30"
        } else {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 text-gray-600 hover:bg-gray-100 hover:text-gray-900"
        }
    });
    let discounts_class = use_memo(move || {
        if current_page() == "discounts" {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 bg-[#e0311f] text-white shadow-lg shadow-[#e0311f]/30"
        } else {
            "px-4 py-2.5 rounded-xl font-medium transition-all duration-200 text-gray-600 hover:bg-gray-100 hover:text-gray-900"
        }
    });
    
    let page_content = use_memo(move || {
        let page = current_page().clone();
        match page.as_str() {
            "floorplan" => rsx! { pages::FloorPlan {} },
            "orders" => rsx! { pages::Orders {} },
            "menu" => rsx! { pages::Menu {} },
            "staff" => rsx! { pages::StaffPage {} },
            "inventory" => rsx! { pages::InventoryPage {} },
            "registry" => rsx! { pages::RegistryPage {} },
            "customers" => rsx! { pages::CustomersPage {} },
            "reservations" => rsx! { pages::ReservationsPage {} },
            "kitchen" => rsx! { pages::KitchenPage {} },
            "reports" => rsx! { pages::ReportsPage {} },
            "login" => rsx! { pages::LoginPage {} },
            "discounts" => rsx! { pages::DiscountsPage {} },
            _ => rsx! { p { "Page not found" } }
        }
    });
    
    rsx! {
        div { class: "min-h-screen bg-gradient-to-br from-gray-50 to-gray-100",
            // Header with modern gradient and glass effect
            header { class: "bg-white/80 backdrop-blur-lg border-b border-gray-200 shadow-sm sticky top-0 z-40",
                div { class: "container mx-auto px-6 py-4",
                    div { class: "flex justify-between items-center",
                        div { class: "flex items-center space-x-3",
                            div { class: "w-10 h-10 bg-gradient-to-br from-[#e0311f] to-[#dc2381] rounded-xl flex items-center justify-center shadow-lg shadow-[#e0311f]/30",
                                span { class: "text-white font-bold text-lg", "K" }
                            }
                            h1 { class: "text-2xl font-bold bg-gradient-to-r from-[#e0311f] to-[#dc2381] bg-clip-text text-transparent", "Kipko POS" }
                        }
                        nav { class: "flex space-x-2",
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
                                class: "{inventory_class}",
                                onclick: move |_| current_page.set("inventory".to_string()),
                                "Inventory"
                            }
                            button {
                                class: "{registry_class}",
                                onclick: move |_| current_page.set("registry".to_string()),
                                "Registry"
                            }
                            button {
                                class: "{customers_class}",
                                onclick: move |_| current_page.set("customers".to_string()),
                                "Customers"
                            }
                            button {
                                class: "{reservations_class}",
                                onclick: move |_| current_page.set("reservations".to_string()),
                                "Reservations"
                            }
                            button {
                                class: "{kitchen_class}",
                                onclick: move |_| current_page.set("kitchen".to_string()),
                                "Kitchen"
                            }
                            button {
                                class: "{reports_class}",
                                onclick: move |_| current_page.set("reports".to_string()),
                                "Reports"
                            }
                            button {
                                class: "{login_class}",
                                onclick: move |_| current_page.set("login".to_string()),
                                "Login"
                            }
                            button {
                                class: "{discounts_class}",
                                onclick: move |_| current_page.set("discounts".to_string()),
                                "Discounts"
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
            
            // Main Content with modern spacing
            main { class: "container mx-auto px-6 py-8",
                {page_content()}
            }
        }
    }
}
