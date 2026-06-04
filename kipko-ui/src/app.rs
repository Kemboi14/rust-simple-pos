use leptos::*;
use leptos_meta::*;
use crate::components::{Navigation, Layout};
use crate::pages::*;

#[component]
pub fn App() -> impl IntoView {
    let current_page = create_rw_signal("floorplan".to_string());

    view! {
        <Stylesheet id="leptos" href="/pkg/kipko-ui.css"/>
        <Title text="Kipko POS"/>
        <Meta name="description" content="Restaurant Point of Sale System"/>

        <Layout>
            <Navigation current_page/>
            <main class="main-content">
                {move || {
                    match current_page.get() {
                        "floorplan" => view! { <FloorPlan/> },
                        "orders" => view! { <Orders/> },
                        "menu" => view! { <Menu/> },
                        "inventory" => view! { <Inventory/> },
                        "customers" => view! { <Customers/> },
                        "staff" => view! { <Staff/> },
                        "reports" => view! { <Reports/> },
                        "accounting" => view! { <Accounting/> },
                        _ => view! { <div>"Page not found"</div> }
                    }
                }}
            </main>
        </Layout>
    }
}