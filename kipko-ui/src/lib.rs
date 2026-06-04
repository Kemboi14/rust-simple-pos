use leptos::*;
use leptos_meta::*;
use leptos_actix::{generate_route_list, LeptosRoutes};

pub mod app;
pub mod components;
pub mod pages;
pub mod services;
pub mod utils;

#[component]
pub fn App() -> impl IntoView {
    // Provides context for metadata
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/kipko-ui.css"/>
        
        // sets the document title
        <Title text="Kipko POS"/>
        
        <main>
            <app::App/>
        </main>
    }
}