//! Kipko POS UI - Simple Working Version

use dioxus::prelude::*;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        div { class: "min-h-screen bg-gray-100",
            header { class: "bg-blue-600 text-white shadow-lg",
                div { class: "container mx-auto px-4 py-4",
                    div { class: "flex justify-between items-center",
                        h1 { class: "text-2xl font-bold", "Kipko POS" }
                        nav { class: "flex space-x-4",
                            button { class: "px-3 py-2 rounded hover:bg-blue-700", "Floor Plan" }
                            button { class: "px-3 py-2 rounded hover:bg-blue-700", "Orders" }
                            button { class: "px-3 py-2 rounded hover:bg-blue-700", "Menu" }
                            button { class: "px-3 py-2 rounded hover:bg-blue-700", "Staff" }
                        }
                    }
                }
            }
            main { class: "container mx-auto px-4 py-8",
                div {
                    h2 { class: "text-3xl font-bold text-gray-900 mb-6", "Floor Plan" }
                    p { class: "text-gray-600 mb-6", "Restaurant floor plan will be displayed here" }
                    div { class: "grid grid-cols-3 gap-4",
                        button { class: "bg-green-500 hover:bg-green-600 text-white font-bold py-4 px-6 rounded-lg", "Table 1" }
                        button { class: "bg-yellow-500 hover:bg-yellow-600 text-white font-bold py-4 px-6 rounded-lg", "Table 2" }
                        button { class: "bg-green-500 hover:bg-green-600 text-white font-bold py-4 px-6 rounded-lg", "Table 3" }
                    }
                }
            }
        }
    }
}
