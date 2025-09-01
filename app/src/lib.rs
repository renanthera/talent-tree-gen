use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use serde::{Deserialize, Serialize};
// use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all(deserialize="camelCase"))]
struct TraitTree {
    trait_tree_id: i32,
    class_id: i32,
    spec_id: i32,
    class_name: String,
    spec_name: String,
}

async fn fetch_trait_trees() -> Result<Vec<TraitTree>, Error> {
    Ok(reqwasm::http::Request::get("/talent-data/talents.json")
        .send()
        .await?
       .json()
    .await?)
}

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

#[component]
fn JsonTest() -> impl IntoView {
    let ttdata = LocalResource::new(move || fetch_trait_trees());
    view! {
        <Transition fallback=|| view! { <div>"Loading..."</div> }>
            <ul>
                {move || Suspend::new(async move {
                    ttdata
                        .await
                        .map(|tt_data| {
                            tt_data
                                .into_iter()
                                .map(|tt| {
                                    // view! { <li>{format!("{tt:?}")}</li> }
                                    view! {
                                        <li>{format!("{0} - {1}", tt.class_name, tt.spec_name)}</li>
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                })}
            </ul>
        </Transition>
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
        <JsonTest />
    }
}
