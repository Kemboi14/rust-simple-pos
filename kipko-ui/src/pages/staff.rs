//! Staff Page

use dioxus::prelude::*;
use kipko_core::{Staff, StaffRole};
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Badge, BadgeVariant};

#[component]
pub fn Staff() -> Element {
    let api = ApiService::new();
    let mut staff = use_resource(move || {
        let api_clone = api.clone();
        async move {
            api_clone.get_staff().await.unwrap_or_default()
        }
    });
    
    let mut selected_staff = use_signal(|| Option::<uuid::Uuid>::None);
    let mut show_details = use_signal(|| false);
    let loading = staff.read().is_none();
    
    let staff_data = use_memo(move || staff.read().clone());

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                h2 { class: "text-2xl font-bold text-gray-900", "Staff" }
                div { class: "flex gap-2",
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| {},
                        children: rsx! { "Add Staff Member" }
                    }
                    Button {
                        variant: ButtonVariant::Secondary,
                        onclick: move |_| { staff.restart(); },
                        children: rsx! { "Refresh" }
                    }
                }
            }

            if loading {
                div { class: "flex justify-center items-center h-64",
                    div { class: "animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600" }
                }
            }

            {let data = staff_data();
            if let Some(staff_list) = data.as_ref() {
                if staff_list.is_empty() {
                    rsx! {
                        div { class: "text-center py-12 p-4 bg-white rounded-lg shadow",
                            p { class: "text-gray-500", "No staff members found" }
                            Button {
                                variant: ButtonVariant::Primary,
                                onclick: move |_| {},
                                children: rsx! { "Add First Staff Member" }
                            }
                        }
                    }
                } else {
                    rsx! {
                        div { class: "grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6",
                            for member in staff_list {
                                rsx! {
                                    div {
                                        class: "cursor-pointer hover:shadow-lg transition-shadow p-4 bg-white rounded-lg shadow",
                                        onclick: move |_| {
                                            selected_staff.set(Some(member.id));
                                            show_details.set(true);
                                        },
                                        StaffCardContent { staff: member.clone() }
                                    }
                                }
                            }
                        }
                    }
                }
            } else { rsx! {} }}

            {let show = *show_details.read();
            let sel_id = *selected_staff.read();
            let data2 = staff_data();
            if show {
                if let Some(selected_id) = sel_id {
                    if let Some(member) = data2.as_ref().and_then(|l| l.iter().find(|s| s.id == selected_id).cloned()) {
                        rsx! {
                            StaffDetails {
                                staff: member,
                                on_close: move |_| {
                                    show_details.set(false);
                                    selected_staff.set(None);
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
fn StaffCardContent(staff: Staff) -> Element {
    let role_variant = match staff.role {
        StaffRole::Manager => BadgeVariant::Primary,
        StaffRole::Admin => BadgeVariant::Danger,
        StaffRole::Server => BadgeVariant::Success,
        StaffRole::Kitchen => BadgeVariant::Warning,
        StaffRole::Host => BadgeVariant::Secondary,
    };
    let role_text = match staff.role {
        StaffRole::Manager => "Manager",
        StaffRole::Admin => "Admin",
        StaffRole::Server => "Server",
        StaffRole::Kitchen => "Kitchen",
        StaffRole::Host => "Host",
    };

    rsx! {
        div { class: "space-y-3",
            div { class: "flex justify-between items-start",
                h3 { class: "text-lg font-semibold text-gray-900", "{staff.name}" }
                if !staff.is_active {
                    Badge { variant: BadgeVariant::Danger, children: rsx! { "Inactive" } }
                }
            }
            p { class: "text-sm text-gray-600", "{staff.email}" }
            Badge { variant: role_variant, children: rsx! { "{role_text}" } }
            div { class: "flex items-center gap-2 text-sm text-gray-600",
                if staff.is_active {
                    span { class: "text-green-600", "● Active" }
                } else {
                    span { class: "text-red-600", "● Inactive" }
                }
            }
        }
    }
}

#[component]
fn StaffDetails(staff: Staff, on_close: EventHandler<MouseEvent>) -> Element {
    let role_variant = match staff.role {
        StaffRole::Manager => BadgeVariant::Primary,
        StaffRole::Admin => BadgeVariant::Danger,
        StaffRole::Server => BadgeVariant::Success,
        StaffRole::Kitchen => BadgeVariant::Warning,
        StaffRole::Host => BadgeVariant::Secondary,
    };
    let role_text = match staff.role {
        StaffRole::Manager => "Manager",
        StaffRole::Admin => "Admin",
        StaffRole::Server => "Server",
        StaffRole::Kitchen => "Kitchen",
        StaffRole::Host => "Host",
    };
    let staff_id = staff.id.to_string();
    let short_id = &staff_id[..8.min(staff_id.len())];

    rsx! {
        div { class: "fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50",
            div { class: "max-w-md w-full mx-4 bg-white rounded-lg shadow p-6",
                div { class: "flex justify-between items-start mb-4",
                    div {
                        h3 { class: "text-xl font-bold text-gray-900", "{staff.name}" }
                        p { class: "text-sm text-gray-500", "ID: {short_id}" }
                    }
                    button { class: "text-gray-400 hover:text-gray-600 text-2xl", onclick: move |e| on_close(e), "×" }
                }
                div { class: "space-y-6",
                    div { class: "flex items-center justify-between p-4 bg-gray-50 rounded-lg",
                        div { class: "space-y-1",
                            div { class: "flex items-center gap-2",
                                span { class: "text-gray-600", "Role:" }
                                Badge { variant: role_variant, children: rsx! { "{role_text}" } }
                            }
                            p { class: "text-sm text-gray-600", "{staff.email}" }
                        }
                    }
                    div { class: "border-t pt-4 flex gap-2",
                        Button { variant: ButtonVariant::Primary, onclick: move |_| {}, children: rsx! { "Edit" } }
                        if staff.is_active {
                            Button { variant: ButtonVariant::Outline, onclick: move |_| {}, children: rsx! { "Deactivate" } }
                        } else {
                            Button { variant: ButtonVariant::Success, onclick: move |_| {}, children: rsx! { "Activate" } }
                        }
                        Button { variant: ButtonVariant::Danger, onclick: move |_| {}, children: rsx! { "Delete" } }
                    }
                }
            }
        }
    }
}
