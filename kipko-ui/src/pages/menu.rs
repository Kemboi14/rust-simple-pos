//! Menu Page

use dioxus::prelude::*;
use kipko_core::{MenuItem, MenuItemCategory};
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Badge, BadgeVariant};

#[component]
pub fn Menu() -> Element {
    let api = ApiService::new();
    let api_clone1 = api.clone();
    let mut categories = use_resource(move || {
        let api_clone = api_clone1.clone();
        async move {
            api_clone.get_menu_categories().await.unwrap_or_default()
        }
    });
    let mut items = use_resource(move || {
        let api_clone = api.clone();
        async move {
            api_clone.get_menu_items().await.unwrap_or_default()
        }
    });
    
    let mut selected_item = use_signal(|| Option::<uuid::Uuid>::None);
    let mut show_details = use_signal(|| false);
    let mut selected_category = use_signal(|| Option::<uuid::Uuid>::None);
    let loading = categories.read().is_none() || items.read().is_none();
    
    let categories_data = use_memo(move || categories.read().clone());
    let items_data = use_memo(move || items.read().clone());

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                h2 { class: "text-2xl font-bold text-gray-900", "Menu" }
                div { class: "flex gap-2",
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| {},
                        children: rsx! { "Add Item" }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| {
                            categories.restart();
                            items.restart();
                        },
                        children: rsx! { "Refresh" }
                    }
                }
            }

            if loading {
                div { class: "flex justify-center items-center h-64",
                    div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" }
                }
            }

            {let cat_data = categories_data();
            if let Some(category_list) = cat_data.as_ref() {
                if category_list.is_empty() {
                    rsx! {
                        div { class: "text-center py-12 p-4 bg-white rounded-lg shadow",
                            p { class: "text-gray-500", "No categories found" }
                        }
                    }
                } else {
                    rsx! {
                        div { class: "flex gap-2 flex-wrap",
                            Button {
                                variant: if selected_category().is_none() { ButtonVariant::Primary } else { ButtonVariant::Outline },
                                onclick: move |_| selected_category.set(None),
                                children: rsx! { "All Categories" }
                            }
                            for category in category_list {
                                Button {
                                    variant: if *selected_category.read() == Some(category.id) { ButtonVariant::Primary } else { ButtonVariant::Outline },
                                    onclick: move |_| selected_category.set(Some(category.id)),
                                    children: rsx! { "{category.name}" }
                                }
                            }
                        }
                    }
                }
            } else { rsx! {} }}

            {let item_data = items_data();
            let cat_data2 = categories_data();
            if let Some(item_list) = item_data.as_ref() {
                if let Some(category_list) = cat_data2.as_ref() {
                    if item_list.is_empty() {
                        rsx! {
                            div { class: "text-center py-12 p-4 bg-white rounded-lg shadow",
                                p { class: "text-gray-500", "No menu items found" }
                            }
                        }
                    } else {
                        rsx! {
                            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                                for item in item_list {
                                    {let category = category_list.iter().find(|c| c.id == item.category_id).cloned();
                                    rsx! {
                                        div {
                                            class: "cursor-pointer hover:shadow-lg transition-shadow p-4 bg-white rounded-lg shadow",
                                            onclick: move |_| {
                                                selected_item.set(Some(item.id));
                                                show_details.set(true);
                                            },
                                            MenuItemCardContent { item: item.clone(), category }
                                        }
                                    }}
                                }
                            }
                        }
                    }
                }
            } else { rsx! {} }}

            {let show = *show_details.read();
            let sel_id = *selected_item.read();
            let item_data2 = items_data();
            let cat_data3 = categories_data();
            if show {
                if let Some(selected_id) = sel_id {
                    if let Some((item, category)) = item_data2.as_ref()
                        .and_then(|l| l.iter().find(|i| i.id == selected_id).cloned())
                        .map(|i| (i, cat_data3.as_ref().and_then(|c| c.iter().find(|cat| cat.id == i.category_id).cloned()))) {
                        rsx! {
                            MenuItemDetails {
                                item: item,
                                category: category,
                                on_close: move |_| {
                                    show_details.set(false);
                                    selected_item.set(None);
                                }
                            }
                        }
                    } else { rsx! {} }
                } else { rsx! {} }
            } else { rsx! {} }}
        }
    }
}

#[component]
fn MenuItemCardContent(item: MenuItem, category: Option<MenuItemCategory>) -> Element {
    rsx! {
        div { class: "space-y-3",
            div { class: "flex justify-between items-start",
                h3 { class: "text-lg font-semibold text-gray-900", "{item.name}" }
                if !item.is_available {
                    Badge { variant: BadgeVariant::Danger, children: rsx! { "Unavailable" } }
                }
            }
            if let Some(desc) = &item.description {
                p { class: "text-sm text-gray-600 line-clamp-2", "{desc}" }
            }
            if let Some(cat) = &category {
                Badge { variant: BadgeVariant::Secondary, children: rsx! { "{cat.name}" } }
            }
            div { class: "flex justify-between items-center pt-2 border-t",
                span { class: "text-2xl font-bold text-gray-900", "${item.price.amount()}" }
                if let Some(prep_time) = item.preparation_time_minutes {
                    span { class: "text-sm text-gray-500", "{prep_time} min" }
                }
            }
        }
    }
}

#[component]
fn MenuItemDetails(
    item: MenuItem,
    category: Option<MenuItemCategory>,
    on_close: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "max-w-lg w-full mx-4 max-h-[90vh] overflow-y-auto bg-white rounded-lg shadow p-6",
                div { class: "flex justify-between items-start mb-4",
                    div {
                        h3 { class: "text-xl font-bold text-gray-900", "{item.name}" }
                        if let Some(cat) = &category {
                            Badge { variant: BadgeVariant::Secondary, children: rsx! { "{cat.name}" } }
                        }
                    }
                    button { class: "text-gray-400 hover:text-gray-600 text-2xl", onclick: move |e| on_close(e), "×" }
                }
                div { class: "space-y-6",
                    div { class: "flex items-center justify-between p-4 bg-gray-50 rounded-lg",
                        div {
                            p { class: "text-3xl font-bold text-gray-900", "${item.price.amount()}" }
                            p { class: "text-sm text-gray-600", "Tax rate: {item.tax_rate}%" }
                        }
                    }
                    if let Some(desc) = &item.description {
                        div {
                            h4 { class: "font-semibold text-gray-900 mb-2", "Description" }
                            p { class: "text-gray-600", "{desc}" }
                        }
                    }
                    div { class: "border-t pt-4 flex gap-2",
                        Button { variant: ButtonVariant::Primary, onclick: move |_| {}, children: rsx! { "Edit" } }
                        Button { variant: ButtonVariant::Danger, onclick: move |_| {}, children: rsx! { "Delete" } }
                    }
                }
            }
        }
    }
}
