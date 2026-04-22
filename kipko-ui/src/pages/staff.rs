//! Staff Page

use dioxus::prelude::*;
use kipko_core::{Staff, StaffRole};
use crate::services::ApiService;
use crate::components::{Button, ButtonVariant, Badge, BadgeVariant};

#[component]
pub fn StaffPage() -> Element {
    let api = ApiService::new();
    let api_clone1 = api.clone();
    let mut staff = use_resource(move || {
        let api_clone = api_clone1.clone();
        async move {
            api_clone.get_staff().await.unwrap_or_default()
        }
    });
    
    let mut selected_staff = use_signal(|| Option::<uuid::Uuid>::None);
    let mut show_details = use_signal(|| false);
    let mut show_add_staff = use_signal(|| false);
    let loading = staff.read().is_none();
    
    let staff_data = use_memo(move || staff.read().clone());

    rsx! {
        div { class: "space-y-6",
            div { class: "flex justify-between items-center",
                div { class: "space-y-1",
                    h2 { class: "text-3xl font-bold bg-gradient-to-r from-[#e0311f] to-[#dc2381] bg-clip-text text-transparent", "Staff" }
                    p { class: "text-gray-500", "Manage your restaurant staff" }
                }
                div { class: "flex gap-3",
                    Button {
                        variant: ButtonVariant::Primary,
                        onclick: move |_| show_add_staff.set(true),
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
                div { class: "flex flex-col items-center justify-center h-64 space-y-4",
                    div { class: "relative w-16 h-16",
                        div { class: "absolute inset-0 border-4 border-gray-200 rounded-full" }
                        div { class: "absolute inset-0 border-4 border-[#e0311f] rounded-full border-t-transparent animate-spin" }
                    }
                    p { class: "text-gray-500 font-medium", "Loading staff..." }
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
                            for member in staff_list.clone() {
                                {let member_id = member.id;
                                rsx! {
                                    div {
                                        class: "cursor-pointer hover:shadow-lg transition-shadow p-4 bg-white rounded-lg shadow",
                                        onclick: move |_| {
                                            selected_staff.set(Some(member_id));
                                            show_details.set(true);
                                        },
                                        StaffCardContent { staff: member.clone() }
                                    }
                                }}
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

            // Add Staff Modal
            {let show = *show_add_staff.read();
            if show {
                let api_clone = api.clone();
                let staff_clone = staff.clone();
                rsx! {
                    AddStaffModal {
                        on_close: move |_| show_add_staff.set(false),
                        on_submit: move |staff_data: CreateStaffData| {
                            let api_clone2 = api_clone.clone();
                            let mut staff_clone2 = staff_clone.clone();
                            dioxus::prelude::spawn(async move {
                                let _ = api_clone2.create_staff(staff_data.name, staff_data.email, staff_data.role).await;
                                staff_clone2.restart();
                            });
                            show_add_staff.set(false);
                        }
                    }
                }
            } else { rsx! {} }}
        }
    }
}

#[derive(Clone)]
pub struct CreateStaffData {
    pub name: String,
    pub email: String,
    pub role: String,
}

#[component]
fn AddStaffModal(on_close: EventHandler<MouseEvent>, on_submit: EventHandler<CreateStaffData>) -> Element {
    let mut name = use_signal(|| String::new());
    let mut email = use_signal(|| String::new());
    let mut role = use_signal(|| "Server".to_string());

    rsx! {
        div { class: "fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50 p-4",
            div { class: "bg-white rounded-3xl shadow-2xl max-w-md w-full mx-4 overflow-hidden",
                // Header
                div { class: "bg-gradient-to-r from-[#e0311f] to-[#dc2381] px-6 py-5",
                    div { class: "flex justify-between items-center",
                        h3 { class: "text-xl font-bold text-white", "Add New Staff Member" }
                        button {
                            class: "w-10 h-10 bg-white/20 hover:bg-white/30 rounded-xl flex items-center justify-center text-white transition-all duration-200 backdrop-blur-sm",
                            onclick: move |e| on_close(e),
                            "×"
                        }
                    }
                }

                div { class: "p-6 space-y-5",
                    // Name
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Full Name" }
                        input {
                            r#type: "text",
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            value: "{name}",
                            oninput: move |e| name.set(e.value())
                        }
                    }

                    // Email
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Email Address" }
                        input {
                            r#type: "email",
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            value: "{email}",
                            oninput: move |e| email.set(e.value())
                        }
                    }

                    // Role
                    div { class: "space-y-2",
                        label { class: "block text-sm font-semibold text-gray-700", "Role" }
                        select {
                            class: "w-full px-4 py-3 rounded-xl border-2 border-gray-200 focus:border-[#e0311f] focus:outline-none transition-colors",
                            onchange: move |e| role.set(e.value()),
                            option { value: "Server", "Server" }
                            option { value: "Manager", "Manager" }
                            option { value: "Kitchen", "Kitchen" }
                            option { value: "Host", "Host" }
                            option { value: "Admin", "Admin" }
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
                                on_submit(CreateStaffData {
                                    name: name(),
                                    email: email(),
                                    role: role(),
                                });
                            },
                            children: rsx! { "Add Staff" }
                        }
                    }
                }
            }
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
