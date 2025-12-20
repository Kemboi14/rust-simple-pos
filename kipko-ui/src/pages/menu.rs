//! Menu Page

use dioxus::prelude::*;

#[component]
pub fn Menu() -> Element {
    rsx! {
        div {
            h2 { "Menu" }
            p { "Menu items will be displayed here" }
            button { "Add Item" }
            div { "Appetizers" }
            div { "Main Courses" }
            div { "Desserts" }
        }
    }
}
