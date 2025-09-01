use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

pub mod trait_tree;

use crate::trait_tree::TraitTreeDebug;

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // sets the document title
        <Title text="WoW Talent Tree Generator" />

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    let (name, set_name) = signal("".to_string());

    view! {
        <button on:click=on_click>"Click Me: " {count}</button>
        <br />
        <input
            type="text"
            on:input:target=move |v| {
                set_name.set(v.target().value());
            }
            prop:value=name
        />
        <p>"Name is: "{name}</p>
        <TraitTreeDebug />
    }
}
