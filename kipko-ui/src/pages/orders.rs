//! Orders Page

use dioxus::prelude::*;
use kipko_core::{Order, OrderStatus, Table, Staff, MenuItem, OrderItem, Payment, Customer};
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Badge, BadgeVariant};

#[component]
pub fn Orders() -> Element {
    let api = ApiService::new();
    let api_clone1 = api.clone();
    let mut orders = use_resource(move || {
        let api_clone = api_clone1.clone();
        async move {
            api_clone.get_orders().await.unwrap_or_default()
        }
    });
    let api_clone2 = api.clone();
    let mut tables = use_resource(move || {
        let api_clone = api_clone2.clone();
        async move {
            api_clone.get_tables().await.unwrap_or_default()
        }
    });
    let api_clone3 = api.clone();
    let mut staff = use_resource(move || {
        let api_clone = api_clone3.clone();
        async move {
            api_clone.get_staff().await.unwrap_or_default()
        }
    });
    let api_clone4 = api.clone();
    let mut menu_items = use_resource(move || {
        let api_clone = api_clone4.clone();
        async move {
            api_clone.get_menu_items().await.unwrap_or_default()
        }
    });

    let mut selected_order = use_signal(|| Option::<uuid::Uuid>::None);
    let mut show_details = use_signal(|| false);
    let mut show_new_order = use_signal(|| false);
    let loading = orders.read().is_none();

    let orders_data = use_memo(move || orders.read().clone());
    let tables_data = use_memo(move || tables.read().clone());
    let staff_data = use_memo(move || staff.read().clone());
    let menu_data = use_memo(move || menu_items.read().clone());

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                div { class: "space-y-1",
                    h2 { class: "text-3xl font-bold bg-gradient-to-r from-[#e0311f] to-[#dc2381] bg-clip-text text-transparent", "Orders" }
                    p { class: "text-gray-500", "Manage customer orders" }
                }
                div { class: "flex gap-3",
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_new_order.set(true),
                        children: rsx! { "New Order" }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| { orders.restart(); },
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
                    p { class: "text-gray-500 font-medium", "Loading orders..." }
                }
            }

            {let data = orders_data();
            if let Some(order_list) = data.as_ref() {
                let list = order_list.clone();
                if list.is_empty() {
                    rsx! {
                        div { class: "text-center py-12 p-6 bg-white rounded-2xl shadow-xl shadow-gray-200/50 border border-gray-100",
                            p { class: "text-gray-500", "No orders found" }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| show_new_order.set(true),
                                children: rsx! { "Create First Order" }
                            }
                        }
                    }
                } else {
                    rsx! {
                        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4",
                            for order in list.clone() {
                                {let order_id = order.id;
                                let sel = selected_order() == Some(order_id);
                                let cls = if sel { "cursor-pointer hover:shadow-lg transition-shadow p-5 bg-white rounded-2xl shadow-xl shadow-gray-200/50 border-2 border-[#e0311f]" } else { "cursor-pointer hover:shadow-lg transition-shadow p-5 bg-white rounded-2xl shadow-xl shadow-gray-200/50 border border-gray-100" };
                                rsx! {
                                    div {
                                        class: "{cls}",
                                        onclick: move |_| {
                                            selected_order.set(Some(order_id));
                                            show_details.set(true);
                                        },
                                        OrderCardContent { order: order.clone() }
                                    }
                                }}
                            }
                        }
                    }
                }
            } else { rsx! {} }}

            {let show = *show_details.read();
            let sel_id = *selected_order.read();
            let data2 = orders_data();
            if show {
                if let Some(selected_id) = sel_id {
                    if let Some(order) = data2.as_ref().and_then(|l| l.iter().find(|o| o.id == selected_id)).cloned() {
                        let api_clone = api.clone();
                        let orders_clone = orders.clone();
                        let menu_clone = menu_data.read().clone();
                        rsx! {
                            OrderDetails {
                                order: order.clone(),
                                menu_items: menu_clone,
                                orders: orders_clone,
                                on_close: move |_| {
                                    show_details.set(false);
                                    selected_order.set(None);
                                },
                                on_close_order: move |_| {
                                    let api_clone2 = api_clone.clone();
                                    let mut orders_clone2 = orders_clone.clone();
                                    dioxus::prelude::spawn(async move {
                                        let _ = api_clone2.close_order(order.id).await;
                                        orders_clone2.restart();
                                    });
                                    show_details.set(false);
                                }
                            }
                        }
                    } else { rsx! {} }
                } else { rsx! {} }
            } else { rsx! {} }}

            // New Order Modal
            {let show = *show_new_order.read();
            if show {
                let api_clone = api.clone();
                let orders_clone = orders.clone();
                let tables_clone = tables_data.read().clone();
                let staff_clone = staff_data.read().clone();
                rsx! {
                    NewOrderModal {
                        on_close: move |_| show_new_order.set(false),
                        on_submit: move |order_data: CreateOrderData| {
                            let api_clone2 = api_clone.clone();
                            let mut orders_clone2 = orders_clone.clone();
                            dioxus::prelude::spawn(async move {
                                let _ = api_clone2.create_order(order_data).await;
                                orders_clone2.restart();
                            });
                            show_new_order.set(false);
                        },
                        tables: tables_clone,
                        staff: staff_clone
                    }
                }
            } else { rsx! {} }}
        }
    }
}

#[component]
fn OrderCardContent(order: Order) -> Element {
    let status_variant = match order.status {
        OrderStatus::Open => BadgeVariant::Success,
        OrderStatus::Closed => BadgeVariant::Secondary,
        OrderStatus::Cancelled => BadgeVariant::Danger,
    };
    let status_text = match order.status {
        OrderStatus::Open => "Open",
        OrderStatus::Closed => "Closed",
        OrderStatus::Cancelled => "Cancelled",
    };
    let order_id = order.id.to_string();
    let short_id = &order_id[..8.min(order_id.len())];
    let table_id = order.table_id.to_string();
    let short_table = &table_id[..8.min(table_id.len())];
    let staff_id = order.staff_id.to_string();
    let short_staff = &staff_id[..8.min(staff_id.len())];

    rsx! {
        div { class: "space-y-3",
            div { class: "flex justify-between items-start",
                div { class: "space-y-1",
                    h3 { class: "text-lg font-bold text-gray-900", "Order #{short_id}" }
                    Badge { variant: status_variant, children: rsx! { "{status_text}" } }
                }
            }
            div { class: "space-y-2 text-sm text-gray-600",
                p { "Table: {short_table}" }
                p { "Staff: {short_staff}" }
            }
            div { class: "flex items-center justify-between pt-2 border-t border-gray-100",
                span { class: "text-sm font-semibold text-gray-900", "KSH {order.total_amount.amount()}" }
                span { class: "text-xs text-gray-500", "Created" }
            }
        }
    }
}

#[component]
fn OrderDetails(
    order: Order,
    menu_items: Option<Vec<MenuItem>>,
    orders: dioxus::prelude::Resource<Vec<Order>>,
    on_close: EventHandler<MouseEvent>,
    on_close_order: EventHandler<MouseEvent>,
) -> Element {
    let api = ApiService::new();
    let api_clone1 = api.clone();
    let mut order_items = use_resource(move || {
        let api_clone = api_clone1.clone();
        async move {
            api_clone.get_order_items(order.id).await.unwrap_or_default()
        }
    });
    let mut show_add_items = use_signal(|| false);
    let mut show_payment = use_signal(|| false);
    let mut selected_menu_item = use_signal(|| Option::<uuid::Uuid>::None);
    let mut item_quantity = use_signal(|| 1);
    let mut item_notes = use_signal(|| String::new());
    let mut payment_method = use_signal(|| "Cash".to_string());

    let status_variant = match order.status {
        OrderStatus::Open => BadgeVariant::Success,
        OrderStatus::Closed => BadgeVariant::Secondary,
        OrderStatus::Cancelled => BadgeVariant::Danger,
    };
    let status_text = match order.status {
        OrderStatus::Open => "Open",
        OrderStatus::Closed => "Closed",
        OrderStatus::Cancelled => "Cancelled",
    };
    let order_id = order.id.to_string();
    let short_id = &order_id[..8.min(order_id.len())];
    let created_at_str = order.created_at.format("%Y-%m-%d %H:%M").to_string();

    rsx! {
        div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
            div { class: "bg-white rounded-3xl shadow-2xl max-w-3xl w-full mx-4 max-h-[90vh] overflow-y-auto",
                // Header
                div { class: "bg-gradient-to-r from-[#e0311f] to-[#dc2381] px-6 py-5",
                    div { class: "flex justify-between items-center",
                        div { class: "space-y-1",
                            h3 { class: "text-xl font-bold text-white", "Order Details" }
                            p { class: "text-sm text-white/80", "ID: {short_id}" }
                        }
                        button {
                            class: "w-10 h-10 bg-white/20 hover:bg-white/30 rounded-xl flex items-center justify-center text-white transition-all duration-200 backdrop-blur-sm",
                            onclick: move |e| on_close(e),
                            "×"
                        }
                    }
                }

                div { class: "p-6 space-y-6",
                    // Status and Total
                    div { class: "flex items-center justify-between p-4 bg-gradient-to-r from-gray-50 to-gray-100 rounded-2xl",
                        div { class: "space-y-1",
                            p { class: "text-sm text-gray-600", "Status" }
                            Badge { variant: status_variant, children: rsx! { "{status_text}" } }
                        }
                        div { class: "text-right space-y-1",
                            p { class: "text-sm text-gray-600", "Total" }
                            p { class: "text-2xl font-bold text-gray-900", "KSH {order.total_amount.amount()}" }
                        }
                    }

                    // Order Items
                    div { class: "space-y-3",
                        h4 { class: "text-lg font-semibold text-gray-900", "Order Items" }
                        {let items = order_items.read();
                        if let Some(item_list) = items.as_ref() {
                            if item_list.is_empty() {
                                rsx! {
                                    p { class: "text-gray-500 italic", "No items in this order" }
                                }
                            } else {
                                rsx! {
                                    div { class: "space-y-2",
                                        for item in item_list {
                                            div { class: "flex justify-between items-center p-3 bg-gray-50 rounded-xl",
                                                div { class: "space-y-1",
                                                    p { class: "font-medium text-gray-900", "Item: {item.menu_item_id}" }
                                                    p { class: "text-sm text-gray-600", "Qty: {item.quantity} × KSH {item.unit_price.amount()}" }
                                                    if let Some(notes) = &item.notes {
                                                        p { class: "text-xs text-gray-500", "{notes}" }
                                                    }
                                                }
                                                p { class: "font-bold text-gray-900", "KSH {item.subtotal().amount()}" }
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            rsx! {
                                div { class: "flex justify-center py-4",
                                    div { class: "animate-spin rounded-full h-8 w-8 border-b-2 border-[#e0311f]" }
                                }
                            }
                        }}
                    }

                    // Order Info
                    div { class: "grid grid-cols-2 gap-4",
                        div { class: "p-4 bg-gradient-to-r from-gray-50 to-gray-100 rounded-2xl",
                            p { class: "text-sm text-gray-600", "Table ID" }
                            p { class: "text-lg font-bold text-gray-900", "{order.table_id}" }
                        }
                        div { class: "p-4 bg-gradient-to-r from-gray-50 to-gray-100 rounded-2xl",
                            p { class: "text-sm text-gray-600", "Staff ID" }
                            p { class: "text-lg font-bold text-gray-900", "{order.staff_id}" }
                        }
                    }

                    // Created At
                    div { class: "p-4 bg-gradient-to-r from-gray-50 to-gray-100 rounded-2xl",
                        p { class: "text-sm text-gray-600", "Created" }
                        p { class: "text-lg font-bold text-gray-900", "{created_at_str}" }
                    }

                    // Actions
                    if matches!(order.status, OrderStatus::Open) {
                        div { class: "border-t border-gray-200 pt-6 flex gap-3 flex-wrap",
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| show_add_items.set(true),
                                children: rsx! { "Add Items" }
                            }
                            Button {
                                variant: ButtonVariant::Success,
                                onclick: move |_| show_payment.set(true),
                                children: rsx! { "Process Payment" }
                            }
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |e| on_close_order(e),
                                children: rsx! { "Close Order" }
                            }
                        }
                    }
                }
            }
        }

        // Add Items Modal
        {let show = *show_add_items.read();
        if show {
            let api_clone = api.clone();
            let order_id = order.id;
            let menu_items_clone = menu_items.clone();
            rsx! {
                AddItemsModal {
                    on_close: move |_| show_add_items.set(false),
                    on_add: EventHandler::new(move |(menu_item_id, quantity, notes)| {
                        let api_clone2 = api_clone.clone();
                        let mut order_items_clone = order_items.clone();
                        dioxus::prelude::spawn(async move {
                            let _ = api_clone2.add_order_item(order_id, menu_item_id, quantity, notes).await;
                            order_items_clone.restart();
                        });
                        show_add_items.set(false);
                    }),
                    menu_items: menu_items_clone
                }
            }
        } else { rsx! {} }}

        // Payment Modal
        {let show = *show_payment.read();
        if show {
            let api_clone = api.clone();
            let order_clone = order.clone();
            let mut orders_clone = orders.clone();
            rsx! {
                PaymentModal {
                    order: order_clone,
                    on_close: move |_| show_payment.set(false),
                    on_payment: move |method| {
                        let api_clone2 = api_clone.clone();
                        let order_id = order.id;
                        let amount = order.total_amount.amount();
                        let mut orders_clone2 = orders_clone.clone();
                        dioxus::prelude::spawn(async move {
                            if let Ok(payment) = api_clone2.create_payment(order_id, amount, method).await {
                                let _ = api_clone2.complete_payment(payment.id, "TXN-".to_string() + &payment.id.to_string()[..8]).await;
                                let _ = api_clone2.close_order(order_id).await;
                                orders_clone2.restart();
                            }
                        });
                        show_payment.set(false);
                    }
                }
            }
        } else { rsx! {} }}
    }
}

#[derive(Clone)]
pub struct CreateOrderData {
    pub table_id: uuid::Uuid,
    pub staff_id: uuid::Uuid,
}

#[component]
fn AddItemsModal(
    on_close: EventHandler<MouseEvent>,
    on_add: EventHandler<(uuid::Uuid, i32, Option<String>)>,
    menu_items: Option<Vec<MenuItem>>,
) -> Element {
    let mut selected_item = use_signal(|| Option::<uuid::Uuid>::None);
    let mut quantity = use_signal(|| 1);
    let mut notes = use_signal(|| String::new());

    rsx! {
        div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
            div { class: "bg-white rounded-3xl shadow-2xl max-w-md w-full mx-4 overflow-hidden",
                // Header
                div { class: "bg-gradient-to-r from-[#55aa86] to-[#3d8a6a] px-6 py-5",
                    div { class: "flex justify-between items-center",
                        h3 { class: "text-xl font-bold text-white", "Add Items to Order" }
                        button {
                            class: "w-10 h-10 bg-white/20 hover:bg-white/30 rounded-xl flex items-center justify-center text-white transition-all duration-200 backdrop-blur-sm",
                            onclick: move |e| on_close(e),
                            "×"
                        }
                    }
                }

                div { class: "p-6 space-y-5",
                    // Menu Item Selection
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Select Menu Item" }
                        select {
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#55aa86] focus:outline-none transition-colors",
                            onchange: move |e| {
                                if let Ok(uuid) = uuid::Uuid::parse_str(&e.value()) {
                                    selected_item.set(Some(uuid));
                                }
                            },
                            option { value: "", "Select an item" }
                            if let Some(items) = &menu_items {
                                for item in items {
                                    option { value: "{item.id}", "{item.name} - KSH {item.price.amount()}" }
                                }
                            }
                        }
                    }

                    // Quantity
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Quantity" }
                        input {
                            r#type: "number",
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#55aa86] focus:outline-none transition-colors",
                            value: "{quantity}",
                            min: "1",
                            oninput: move |e| {
                                if let Ok(q) = e.value().parse::<i32>() {
                                    quantity.set(q.max(1));
                                }
                            }
                        }
                    }

                    // Notes
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Notes (optional)" }
                        textarea {
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#55aa86] focus:outline-none transition-colors resize-none",
                            rows: 2,
                            placeholder: "Special instructions...",
                            value: "{notes}",
                            oninput: move |e| notes.set(e.value())
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
                                if let Some(item_id) = selected_item() {
                                    on_add((item_id, quantity(), if notes().is_empty() { None } else { Some(notes()) }));
                                }
                            },
                            disabled: selected_item().is_none(),
                            children: rsx! { "Add Item" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PaymentModal(
    order: Order,
    on_close: EventHandler<MouseEvent>,
    on_payment: EventHandler<String>,
) -> Element {
    let mut payment_method = use_signal(|| "Cash".to_string());

    rsx! {
        div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
            div { class: "bg-white rounded-3xl shadow-2xl max-w-md w-full mx-4 overflow-hidden",
                // Header
                div { class: "bg-gradient-to-r from-[#55aa86] to-[#3d8a6a] px-6 py-5",
                    div { class: "flex justify-between items-center",
                        h3 { class: "text-xl font-bold text-white", "Process Payment" }
                        button {
                            class: "w-10 h-10 bg-white/20 hover:bg-white/30 rounded-xl flex items-center justify-center text-white transition-all duration-200 backdrop-blur-sm",
                            onclick: move |e| on_close(e),
                            "×"
                        }
                    }
                }

                div { class: "p-6 space-y-5",
                    // Order Total
                    div { class: "p-4 bg-gradient-to-r from-gray-50 to-gray-100 rounded-2xl text-center",
                        p { class: "text-sm text-gray-600", "Total Amount" }
                        p { class: "text-3xl font-bold text-gray-900", "KSH {order.total_amount.amount()}" }
                    }

                    // Payment Method
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Payment Method" }
                        select {
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#55aa86] focus:outline-none transition-colors",
                            onchange: move |e| payment_method.set(e.value()),
                            option { value: "Cash", "Cash" }
                            option { value: "Card", "Card" }
                            option { value: "MobileMoney", "Mobile Money" }
                            option { value: "Mpesa", "M-Pesa" }
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
                            variant: ButtonVariant::Success,
                            onclick: move |_| {
                                on_payment(payment_method());
                            },
                            children: rsx! { "Process Payment" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn NewOrderModal(
    on_close: EventHandler<MouseEvent>,
    on_submit: EventHandler<CreateOrderData>,
    tables: Option<Vec<Table>>,
    staff: Option<Vec<Staff>>,
) -> Element {
    let mut selected_table = use_signal(|| Option::<uuid::Uuid>::None);
    let mut selected_staff = use_signal(|| Option::<uuid::Uuid>::None);

    rsx! {
        div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
            div { class: "bg-white rounded-3xl shadow-2xl max-w-md w-full mx-4 overflow-hidden",
                // Header
                div { class: "bg-gradient-to-r from-[#e0311f] to-[#dc2381] px-6 py-5",
                    div { class: "flex justify-between items-center",
                        h3 { class: "text-xl font-bold text-white", "Create New Order" }
                        button {
                            class: "w-10 h-10 bg-white/20 hover:bg-white/30 rounded-xl flex items-center justify-center text-white transition-all duration-200 backdrop-blur-sm",
                            onclick: move |e| on_close(e),
                            "×"
                        }
                    }
                }

                div { class: "p-6 space-y-5",
                    // Table Selection
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Select Table" }
                        select {
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            onchange: move |e| {
                                if let Ok(uuid) = uuid::Uuid::parse_str(&e.value()) {
                                    selected_table.set(Some(uuid));
                                }
                            },
                            option { value: "", "Select a table" }
                            if let Some(tables) = &tables {
                                for table in tables {
                                    option { value: "{table.id}", "Table {table.number} - {table.location.as_deref().unwrap_or(\"Unknown\")}" }
                                }
                            }
                        }
                    }

                    // Staff Selection
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Select Staff" }
                        select {
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            onchange: move |e| {
                                if let Ok(uuid) = uuid::Uuid::parse_str(&e.value()) {
                                    selected_staff.set(Some(uuid));
                                }
                            },
                            option { value: "", "Select staff member" }
                            if let Some(staff_list) = &staff {
                                for staff_member in staff_list {
                                    option { value: "{staff_member.id}", "{staff_member.name}" }
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
                                if let (Some(table_id), Some(staff_id)) = (selected_table(), selected_staff()) {
                                    on_submit(CreateOrderData {
                                        table_id,
                                        staff_id,
                                    });
                                }
                            },
                            disabled: selected_table().is_none() || selected_staff().is_none(),
                            children: rsx! { "Create Order" }
                        }
                    }
                }
            }
        }
    }
}
