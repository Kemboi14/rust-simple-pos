use leptos::*;

#[component]
pub fn Layout(children: Children) -> impl IntoView {
    view! {
        <div class="app-layout">
            {children()}
        </div>
    }
}