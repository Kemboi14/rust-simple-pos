//! Layout components
//! 
//! This module contains layout components used throughout the application.

use dioxus::prelude::*;

#[component]
pub fn Header(
    title: String,
    current_page: String,
    on_page_change: EventHandler<String>,
) -> Element {
    rsx! {
        header { class: "bg-blue-600 text-white shadow-lg",
            div { class: "container mx-auto px-4 py-4",
                div { class: "flex items-center justify-between",
                    h1 { class: "text-2xl font-bold", "{title}" }
                    nav { class: "flex space-x-4",
                        button {
                            class: "px-4 py-2 rounded hover:bg-blue-700",
                            onclick: move |_| on_page_change.call("floorplan".to_string()),
                            "Floor Plan"
                        }
                        button {
                            class: "px-4 py-2 rounded hover:bg-blue-700",
                            onclick: move |_| on_page_change.call("orders".to_string()),
                            "Orders"
                        }
                        button {
                            class: "px-4 py-2 rounded hover:bg-blue-700",
                            onclick: move |_| on_page_change.call("menu".to_string()),
                            "Menu"
                        }
                        button {
                            class: "px-4 py-2 rounded hover:bg-blue-700",
                            onclick: move |_| on_page_change.call("staff".to_string()),
                            "Staff"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn Sidebar(
    on_page_change: EventHandler<String>,
    children: Element,
) -> Element {
    rsx! {
        div { class: "flex h-screen bg-gray-100",
            // Sidebar
            aside { class: "w-64 bg-white shadow-md",
                div { class: "p-6",
                    h2 { class: "text-xl font-bold text-gray-800", "Kipko POS" }
                }
                nav { class: "mt-6",
                    ul { class: "space-y-2",
                        li {
                            button {
                                class: "w-full text-left px-4 py-2 rounded hover:bg-gray-100",
                                onclick: move |_| on_page_change.call("floorplan".to_string()),
                                "Floor Plan"
                            }
                        }
                        li {
                            button {
                                class: "w-full text-left px-4 py-2 rounded hover:bg-gray-100",
                                onclick: move |_| on_page_change.call("orders".to_string()),
                                "Orders"
                            }
                        }
                        li {
                            button {
                                class: "w-full text-left px-4 py-2 rounded hover:bg-gray-100",
                                onclick: move |_| on_page_change.call("menu".to_string()),
                                "Menu"
                            }
                        }
                        li {
                            button {
                                class: "w-full text-left px-4 py-2 rounded hover:bg-gray-100",
                                onclick: move |_| on_page_change.call("staff".to_string()),
                                "Staff"
                            }
                        }
                    }
                }
            }
            
            // Main content
            main { class: "flex-1 overflow-y-auto",
                {children}
            }
        }
    }
}

#[component]
pub fn Container(
    children: Element,
    size: Option<String>,
) -> Element {
    let size_classes = match size.unwrap_or_default().as_str() {
        "small" => "max-w-2xl",
        "large" => "max-w-6xl",
        _ => "max-w-4xl",
    };
    
    rsx! {
        div { class: "mx-auto px-4 sm:px-6 lg:px-8 {size_classes}",
            {children}
        }
    }
}

#[component]
pub fn Grid(
    children: Element,
    cols: Option<u32>,
    gap: Option<String>,
    class: Option<String>,
) -> Element {
    let cols_classes = match cols.unwrap_or(1) {
        1 => "grid-cols-1",
        2 => "grid-cols-1 md:grid-cols-2",
        3 => "grid-cols-1 md:grid-cols-2 lg:grid-cols-3",
        4 => "grid-cols-1 md:grid-cols-2 lg:grid-cols-4",
        6 => "grid-cols-2 md:grid-cols-3 lg:grid-cols-6",
        8 => "grid-cols-2 md:grid-cols-4 lg:grid-cols-8",
        _ => "grid-cols-1",
    };
    
    let gap_class = gap.unwrap_or("gap-4".to_string());
    let additional_class = class.unwrap_or_default();
    
    rsx! {
        div { class: "grid {cols_classes} {gap_class} {additional_class}",
            {children}
        }
    }
}

#[component]
pub fn Flex(
    children: Element,
    direction: Option<FlexDirection>,
    align: Option<FlexAlign>,
    justify: Option<FlexJustify>,
    gap: Option<String>,
    class: Option<String>,
) -> Element {
    let direction_class = match direction.unwrap_or(FlexDirection::Row) {
        FlexDirection::Row => "flex-row",
        FlexDirection::Col => "flex-col",
        FlexDirection::RowReverse => "flex-row-reverse",
        FlexDirection::ColReverse => "flex-col-reverse",
    };
    
    let align_class = match align.unwrap_or(FlexAlign::Start) {
        FlexAlign::Start => "items-start",
        FlexAlign::Center => "items-center",
        FlexAlign::End => "items-end",
        FlexAlign::Stretch => "items-stretch",
        FlexAlign::Baseline => "items-baseline",
    };
    
    let justify_class = match justify.unwrap_or(FlexJustify::Start) {
        FlexJustify::Start => "justify-start",
        FlexJustify::Center => "justify-center",
        FlexJustify::End => "justify-end",
        FlexJustify::Between => "justify-between",
        FlexJustify::Around => "justify-around",
        FlexJustify::Evenly => "justify-evenly",
    };
    
    let gap_class = gap.unwrap_or_default();
    let additional_class = class.unwrap_or_default();
    
    rsx! {
        div { class: "flex {direction_class} {align_class} {justify_class} {gap_class} {additional_class}",
            {children}
        }
    }
}

#[derive(Clone, PartialEq)]
pub enum FlexDirection {
    Row,
    Col,
    RowReverse,
    ColReverse,
}

#[derive(Clone, PartialEq)]
pub enum FlexAlign {
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}

#[derive(Clone, PartialEq)]
pub enum FlexJustify {
    Start,
    Center,
    End,
    Between,
    Around,
    Evenly,
}
