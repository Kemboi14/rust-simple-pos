//! Floor Plan Page

use dioxus::prelude::*;
use kipko_core::{Table, TableStatus};

#[component]
pub fn FloorPlan() -> Element {
    rsx! {
        div {
            h2 { "Floor Plan" }
            p { "Restaurant floor plan will be displayed here" }
            button { "Table 1" }
            button { "Table 2" }
            button { "Table 3" }
        }
    }
}

#[component]
fn TableComponent(table: Table, selected: bool, onclick: EventHandler<MouseEvent>) -> Element {
    let status_color = match table.status {
        TableStatus::Empty => "bg-green-500 hover:bg-green-600",
        TableStatus::Occupied => "bg-yellow-500 hover:bg-yellow-600",
        TableStatus::Dirty => "bg-red-500 hover:bg-red-600",
        TableStatus::Reserved => "bg-gray-500 hover:bg-gray-600",
    };

    let selected_class = if selected { "ring-2 ring-blue-800 ring-offset-2" } else { "" };

    rsx! {
        button {
            class: "w-16 h-16 rounded-lg text-white font-semibold text-xs transition-colors {status_color} {selected_class}",
            onclick: move |e| onclick(e),
            "T{table.number}"
        }
    }
}
