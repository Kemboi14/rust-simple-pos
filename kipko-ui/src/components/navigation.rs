use leptos::*;

#[component]
pub fn Navigation(current_page: RwSignal<String>) -> impl IntoView {
    let pages = vec![
        ("floorplan", "Floor Plan", "#e0311f"),
        ("orders", "Orders", "#55aa86"),
        ("menu", "Menu", "#dc2381"),
        ("inventory", "Inventory", "#f77f00"),
        ("customers", "Customers", "#2a9d8f"),
        ("staff", "Staff", "#e0311f"),
        ("reports", "Reports", "#7209b7"),
        ("accounting", "Accounting", "#f48c06"),
    ];

    view! {
        <nav class="navigation">
            <div class="nav-header">
                <div class="logo">
                    <span class="logo-icon">"K"</span>
                    <h1 class="logo-text">"Kipko POS"</h1>
                </div>
            </div>
            <div class="nav-links">
                {pages.into_iter().map(|(id, label, color)| {
                    let page = id.to_string();
                    let current = current_page.clone();
                    let is_active = move || current.get() == page;
                    
                    view! {
                        <button
                            class=move || format!("nav-link {}", if is_active() { "active" } else { "" })
                            style=move || format!(
                                "--active-color: {}; --hover-color: {};",
                                if is_active() { color } else { "transparent" },
                                color
                            )
                            on:click=move |_| current_page.set(page.clone())
                        >
                            {label}
                        </button>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </nav>
    }
}