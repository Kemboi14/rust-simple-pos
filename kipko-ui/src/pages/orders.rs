//! Orders Page

use dioxus::prelude::*;

#[component]
pub fn Orders() -> Element {
    rsx! {
        div {
            h2 { "Orders" }
            p { "Order management will be displayed here" }
            button { "New Order" }
            div { "Order 1" }
            div { "Order 2" }
        }
    }
}
