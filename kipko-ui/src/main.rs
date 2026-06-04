use leptos::*;
use leptos_actix::{get_server_context, LeptosRoutes};
use leptos_meta::{provide_meta_context, Meta};
use app::App;

#[cfg(feature = "ssr")]
use actix_web::{web, App, HttpResponse, HttpServer};
#[cfg(feature = "ssr")]
use actix_web::dev::Server;

#[component]
pub fn App() -> impl IntoView {
    // Provides context for metadata
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/kipko-ui.css"/>
        <Title text="Kipko POS"/>
        
        <main class="app-container">
            <App/>
        </main>
    }
}

#[cfg(feature = "hydrate")]
fn main() {
    console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    
    leptos::mount_to_body(App);
}

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let server_addr = "127.0.0.1:3000";
    log::info!("Starting server at http://{}", server_addr);
    
    HttpServer::new(move || {
        App::new()
            .route("/api/{tail:.*}", leptos_actix::handle_server_fns())
            .leptos_routes(leptos_actix::RouteConfig::default(), App, {
                leptos_actix::generate_route_list(App)
            })
            .route("/", leptos_actix::render_file_to_stream("index.html"))
    })
    .bind(server_addr)?
    .run()
    .await
}