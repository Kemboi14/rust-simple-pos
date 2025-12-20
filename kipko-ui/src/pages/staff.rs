//! Staff Page

use dioxus::prelude::*;

#[component]
pub fn Staff() -> Element {
    rsx! {
        div {
            h2 { "Staff" }
            p { "Staff management will be displayed here" }
            button { "Add Staff Member" }
            div { "John Doe - Manager" }
            div { "Jane Smith - Server" }
            div { "Mike Johnson - Kitchen" }
        }
    }
}
