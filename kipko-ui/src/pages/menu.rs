//! Menu Page

use dioxus::prelude::*;
use kipko_core::{MenuItem, MenuItemCategory};
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Badge, BadgeVariant};

#[component]
pub fn Menu() -> Element {
    let api = ApiService::new();
    let api_clone1 = api.clone();
    let api_clone2 = api.clone();
    let mut categories = use_resource(move || {
        let api_clone = api_clone1.clone();
        async move {
            api_clone.get_menu_categories().await.unwrap_or_default()
        }
    });
    let mut items = use_resource(move || {
        let api_clone = api_clone2.clone();
        async move {
            api_clone.get_menu_items().await.unwrap_or_default()
        }
    });
    
    let mut selected_item = use_signal(|| Option::<uuid::Uuid>::None);
    let mut show_details = use_signal(|| false);
    let mut show_add_item = use_signal(|| false);
    let mut selected_category = use_signal(|| Option::<uuid::Uuid>::None);
    let loading = categories.read().is_none() || items.read().is_none();
    
    let categories_data = use_memo(move || categories.read().clone());
    let items_data = use_memo(move || items.read().clone());

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                div { class: "space-y-1",
                    h2 { class: "text-3xl font-bold bg-gradient-to-r from-[#e0311f] to-[#dc2381] bg-clip-text text-transparent", "Menu" }
                    p { class: "text-gray-500", "Manage your restaurant menu items" }
                }
                div { class: "flex gap-3",
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_add_item.set(true),
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
                div { class: "flex flex-col items-center justify-center h-64 space-y-4",
                    div { class: "relative w-16 h-16",
                        div { class: "absolute inset-0 border-4 border-gray-200 rounded-full" }
                        div { class: "absolute inset-0 border-4 border-[#e0311f] rounded-full border-t-transparent animate-spin" }
                    }
                    p { class: "text-gray-500 font-medium", "Loading menu..." }
                }
            }

            {let cat_data = categories_data();
            if let Some(category_list) = cat_data.as_ref() {
                let cats = category_list.clone();
                if cats.is_empty() {
                    rsx! {
                        div { class: "text-center py-12 p-6 bg-white rounded-2xl shadow-xl shadow-slate-200/50 border border-slate-100",
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
                            for category in cats.clone() {
                                {let cat_id = category.id;
                                rsx! {
                                    Button {
                                        variant: if selected_category() == Some(cat_id) { ButtonVariant::Primary } else { ButtonVariant::Outline },
                                        onclick: move |_| selected_category.set(Some(cat_id)),
                                        children: rsx! { "{category.name}" }
                                    }
                                }}
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
                            div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6",
                                for item in item_list.clone() {
                                    {let item_id = item.id;
                                    let category = category_list.iter().find(|c| c.id == item.category_id).cloned();
                                    rsx! {
                                        div {
                                            class: "cursor-pointer hover:shadow-2xl transition-all duration-300 transform hover:-translate-y-1 p-5 bg-white rounded-2xl shadow-xl shadow-slate-200/50 border border-slate-100",
                                            onclick: move |_| {
                                                selected_item.set(Some(item_id));
                                                show_details.set(true);
                                            },
                                            MenuItemCardContent { item: item.clone(), category }
                                        }
                                    }}
                                }
                            }
                        }
                    }
                } else { rsx! {} }
            } else { rsx! {} }}

            {let show = *show_details.read();
            let sel_id = *selected_item.read();
            let item_data2 = items_data();
            let cat_data3 = categories_data();
            if show {
                if let Some(selected_id) = sel_id {
                    let maybe_item = item_data2.as_ref()
                        .and_then(|l| l.iter().find(|i| i.id == selected_id).cloned());
                    if let Some(item) = maybe_item {
                        let category = cat_data3.as_ref()
                            .and_then(|c| c.iter().find(|cat| cat.id == item.category_id).cloned());
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

            // Add Item Modal
            {let show = *show_add_item.read();
            if show {
                let api_clone = api.clone();
                let items_clone = items.clone();
                let categories_clone = categories.clone();
                rsx! {
                    AddMenuItemModal {
                        on_close: move |_| show_add_item.set(false),
                        on_submit: move |item_data: CreateMenuItemData| {
                            let api_clone2 = api_clone.clone();
                            let mut items_clone2 = items_clone.clone();
                            dioxus::prelude::spawn(async move {
                                let _ = api_clone2.create_menu_item(item_data).await;
                                items_clone2.restart();
                            });
                            show_add_item.set(false);
                        },
                        categories: categories_clone.read().clone()
                    }
                }
            } else { rsx! {} }}
        }
    }
}

#[component]
fn MenuItemCardContent(item: MenuItem, category: Option<MenuItemCategory>) -> Element {
    let stock_status = if item.stock_quantity == 0 {
        ("Out of Stock", "bg-rose-100 text-rose-800 border-rose-200")
    } else if item.stock_quantity <= item.low_stock_threshold {
        ("Low Stock", "bg-amber-100 text-amber-800 border-amber-200")
    } else {
        ("In Stock", "bg-emerald-100 text-emerald-800 border-emerald-200")
    };

    rsx! {
        div { class: "space-y-4",
            // Product Image
            if let Some(image_url) = &item.image_url {
                div { class: "w-full h-40 bg-gradient-to-br from-slate-100 to-slate-200 rounded-xl overflow-hidden",
                    img {
                        src: "{image_url}",
                        alt: "{item.name}",
                        class: "w-full h-full object-cover"
                    }
                }
            } else {
                div { class: "w-full h-40 bg-gradient-to-br from-gray-100 to-gray-200 rounded-xl flex items-center justify-center",
                    span { class: "text-4xl", "🍽️" }
                }
            }

            // Item Info
            div { class: "space-y-2",
                div { class: "flex justify-between items-start",
                    h3 { class: "text-lg font-bold text-gray-900", "{item.name}" }
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
            }

            // Stock Status
            div { class: "flex items-center justify-between px-3 py-2 rounded-xl border-2 {stock_status.1}",
                span { class: "text-sm font-semibold", "{stock_status.0}" }
                span { class: "text-sm font-bold", "{item.stock_quantity} units" }
            }

            // Price and Details
            div { class: "flex justify-between items-center pt-3 border-t border-slate-200",
                div { class: "space-y-1",
                    span { class: "text-2xl font-bold bg-gradient-to-r from-[#e0311f] to-[#dc2381] bg-clip-text text-transparent", "KSH {item.price.amount()}" }
                    p { class: "text-xs text-gray-500", "Tax: {item.tax_rate}%" }
                }
                if let Some(prep_time) = item.preparation_time_minutes {
                    div { class: "flex items-center space-x-1 text-sm text-gray-500",
                        span { "⏱️" }
                        span { "{prep_time} min" }
                    }
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
    let stock_status = if item.stock_quantity == 0 {
        ("Out of Stock", "bg-rose-100 text-rose-800 border-rose-200")
    } else if item.stock_quantity <= item.low_stock_threshold {
        ("Low Stock", "bg-amber-100 text-amber-800 border-amber-200")
    } else {
        ("In Stock", "bg-emerald-100 text-emerald-800 border-emerald-200")
    };

    rsx! {
        div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
            div { class: "bg-white rounded-3xl shadow-2xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto",
                // Header with gradient
                div { class: "bg-gradient-to-r from-indigo-500 to-purple-600 px-6 py-5",
                    div { class: "flex justify-between items-center",
                        div { class: "space-y-1",
                            h3 { class: "text-2xl font-bold text-white", "{item.name}" }
                            if let Some(cat) = &category {
                                Badge { variant: BadgeVariant::Secondary, children: rsx! { "{cat.name}" } }
                            }
                        }
                        button {
                            class: "w-10 h-10 bg-white/20 hover:bg-white/30 rounded-xl flex items-center justify-center text-white transition-all duration-200 backdrop-blur-sm",
                            onclick: move |e| on_close(e),
                            "×"
                        }
                    }
                }

                div { class: "p-6 space-y-6",
                    // Product Image
                    if let Some(image_url) = &item.image_url {
                        div { class: "w-full h-64 bg-gradient-to-br from-slate-100 to-slate-200 rounded-2xl overflow-hidden",
                            img {
                                src: "{image_url}",
                                alt: "{item.name}",
                                class: "w-full h-full object-cover"
                            }
                        }
                    } else {
                        div { class: "w-full h-64 bg-gradient-to-br from-gray-100 to-gray-200 rounded-2xl flex items-center justify-center",
                            span { class: "text-6xl", "🍽️" }
                        }
                    }

                    // Price and Tax
                    div { class: "flex items-center justify-between p-5 bg-gradient-to-r from-gray-50 to-gray-100 rounded-2xl",
                        div { class: "space-y-1",
                            p { class: "text-4xl font-bold bg-gradient-to-r from-[#e0311f] to-[#dc2381] bg-clip-text text-transparent", "KSH {item.price.amount()}" }
                            p { class: "text-sm text-gray-600", "Tax rate: {item.tax_rate}%" }
                        }
                        if let Some(prep_time) = item.preparation_time_minutes {
                            div { class: "flex items-center space-x-2 text-gray-600",
                                span { class: "text-2xl", "⏱️" }
                                span { class: "text-lg font-semibold", "{prep_time} min" }
                            }
                        }
                    }

                    // Stock Status
                    div { class: "flex items-center justify-between p-4 bg-gradient-to-r from-gray-50 to-gray-100 rounded-2xl border-2 {stock_status.1}",
                        div { class: "flex items-center space-x-3",
                            span { class: "text-2xl", if item.stock_quantity > 0 { "📦" } else { "🚫" } }
                            div { class: "space-y-1",
                                p { class: "text-sm font-semibold text-gray-600", "Stock Status" }
                                p { class: "text-lg font-bold text-gray-900", "{stock_status.0}" }
                            }
                        }
                        div { class: "text-right space-y-1",
                            p { class: "text-sm text-gray-600", "Quantity" }
                            p { class: "text-2xl font-bold text-gray-900", "{item.stock_quantity}" }
                        }
                    }

                    // Description
                    if let Some(desc) = &item.description {
                        div { class: "space-y-2",
                            h4 { class: "font-bold text-gray-900 flex items-center space-x-2",
                                span { class: "text-xl", "📝" }
                                span { "Description" }
                            }
                            p { class: "text-gray-600 leading-relaxed", "{desc}" }
                        }
                    }

                    // Availability Status
                    div { class: "flex items-center justify-between p-4 bg-gradient-to-r from-gray-50 to-gray-100 rounded-2xl",
                        span { class: "text-gray-600 font-medium", "Availability" }
                        if item.is_available {
                            Badge { variant: BadgeVariant::Success, children: rsx! { "Available" } }
                        } else {
                            Badge { variant: BadgeVariant::Danger, children: rsx! { "Unavailable" } }
                        }
                    }

                    // Actions
                    div { class: "border-t border-gray-200 pt-6 space-y-4",
                        div { class: "grid grid-cols-2 gap-3",
                            Button { variant: ButtonVariant::Primary, onclick: move |_| {}, children: rsx! { "Edit Item" } }
                            Button { variant: ButtonVariant::Danger, onclick: move |_| {}, children: rsx! { "Delete Item" } }
                        }
                        // Stock Adjustment
                        div { class: "flex items-center gap-3 p-4 bg-gradient-to-r from-gray-50 to-gray-100 rounded-2xl",
                            button {
                                class: "w-10 h-10 bg-[#e0311f] hover:bg-[#c4211a] text-white rounded-xl flex items-center justify-center font-bold text-xl transition-colors",
                                onclick: move |_| {},
                                "-"
                            }
                            span { class: "text-lg font-bold text-gray-900 flex-1 text-center", "{item.stock_quantity}" }
                            button {
                                class: "w-10 h-10 bg-[#55aa86] hover:bg-[#4a9a76] text-white rounded-xl flex items-center justify-center font-bold text-xl transition-colors",
                                onclick: move |_| {},
                                "+"
                            }
                        }
                        p { class: "text-xs text-gray-500 text-center", "Adjust stock quantity" }
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct CreateMenuItemData {
    pub category_id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub price: rust_decimal::Decimal,
    pub tax_rate: rust_decimal::Decimal,
    pub image_url: String,
    pub stock_quantity: i32,
    pub low_stock_threshold: i32,
}

#[component]
fn AddMenuItemModal(
    on_close: EventHandler<MouseEvent>,
    on_submit: EventHandler<CreateMenuItemData>,
    categories: Option<Vec<MenuItemCategory>>,
) -> Element {
    let mut name = use_signal(|| String::new());
    let mut description = use_signal(|| String::new());
    let mut price = use_signal(|| rust_decimal::Decimal::from(0));
    let mut tax_rate = use_signal(|| rust_decimal::Decimal::from(16));
    let mut image_url = use_signal(|| String::new());
    let mut stock_quantity = use_signal(|| 0);
    let mut low_stock_threshold = use_signal(|| 10);
    let mut selected_category = use_signal(|| Option::<uuid::Uuid>::None);

    rsx! {
        div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
            div { class: "bg-white rounded-3xl shadow-2xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto",
                // Header
                div { class: "bg-gradient-to-r from-[#e0311f] to-[#dc2381] px-6 py-5",
                    div { class: "flex justify-between items-center",
                        h3 { class: "text-xl font-bold text-white", "Add New Menu Item" }
                        button {
                            class: "w-10 h-10 bg-white/20 hover:bg-white/30 rounded-xl flex items-center justify-center text-white transition-all duration-200 backdrop-blur-sm",
                            onclick: move |e| on_close(e),
                            "×"
                        }
                    }
                }

                div { class: "p-6 space-y-5",
                    // Category Selection
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Category" }
                        select {
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            onchange: move |e| {
                                if let Ok(uuid) = uuid::Uuid::parse_str(&e.value()) {
                                    selected_category.set(Some(uuid));
                                }
                            },
                            option { value: "", "Select a category" }
                            if let Some(cats) = &categories {
                                for cat in cats {
                                    option { value: "{cat.id}", "{cat.name}" }
                                }
                            }
                        }
                    }

                    // Name
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Item Name" }
                        input {
                            r#type: "text",
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            value: "{name}",
                            oninput: move |e| name.set(e.value())
                        }
                    }

                    // Description
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Description" }
                        textarea {
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors resize-none",
                            rows: 3,
                            value: "{description}",
                            oninput: move |e| description.set(e.value())
                        }
                    }

                    // Price
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Price (KSH)" }
                        input {
                            r#type: "number",
                            step: "0.01",
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            value: "{price}",
                            oninput: move |e| {
                                if let Ok(p) = e.value().parse() {
                                    price.set(p);
                                }
                            }
                        }
                    }

                    // Tax Rate
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Tax Rate (%)" }
                        input {
                            r#type: "number",
                            step: "0.1",
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            value: "{tax_rate}",
                            oninput: move |e| {
                                if let Ok(t) = e.value().parse() {
                                    tax_rate.set(t);
                                }
                            }
                        }
                    }

                    // Image URL
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Image URL (from Unsplash, etc.)" }
                        input {
                            r#type: "url",
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            placeholder: "https://images.unsplash.com/...",
                            value: "{image_url}",
                            oninput: move |e| image_url.set(e.value())
                        }
                        p { class: "text-xs text-gray-500", "Paste an image URL from Unsplash, Pexels, or any online source" }
                    }

                    // Stock Quantity
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Initial Stock Quantity" }
                        input {
                            r#type: "number",
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            value: "{stock_quantity}",
                            oninput: move |e| {
                                if let Ok(q) = e.value().parse() {
                                    stock_quantity.set(q);
                                }
                            }
                        }
                    }

                    // Low Stock Threshold
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Low Stock Threshold" }
                        input {
                            r#type: "number",
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            value: "{low_stock_threshold}",
                            oninput: move |e| {
                                if let Ok(t) = e.value().parse() {
                                    low_stock_threshold.set(t);
                                }
                            }
                        }
                    }

                    // Actions
                    div { class: "flex gap-3 pt-4",
                        Button {
                            variant: ButtonVariant::Outline,
                            onclick: move |e| on_close(e),
                            children: rsx! { "Cancel" }
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            onclick: move |_| {
                                if let Some(cat_id) = selected_category() {
                                    on_submit(CreateMenuItemData {
                                        category_id: cat_id,
                                        name: name(),
                                        description: description(),
                                        price: price(),
                                        tax_rate: tax_rate(),
                                        image_url: image_url(),
                                        stock_quantity: stock_quantity(),
                                        low_stock_threshold: low_stock_threshold(),
                                    });
                                }
                            },
                            disabled: selected_category().is_none(),
                            children: rsx! { "Add Item" }
                        }
                    }
                }
            }
        }
    }
}
