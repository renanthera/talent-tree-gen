use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

mod defaults;
mod talent_string;
mod trait_tree;
mod trait_types;

use crate::trait_types::{TalentConfiguration, TalentParseError};

use crate::talent_string::TalentConfigView;
use crate::trait_tree::fetch_trait_trees;
use crate::trait_tree::TraitTreeDebug;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Title text="WoW Talent Tree Generator" />
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}
/*
1. use default version if no version is selected
   - otherwise, load version data
2. load spec data for version x spec
*/

#[component]
fn HomePage() -> impl IntoView {
    let (talent_str, set_talent_str) =
        signal::<Result<TalentConfiguration, TalentParseError>>(Err(TalentParseError::NoString));

    let trait_tree_data = LocalResource::new(move || fetch_trait_trees());

    view! {
        <input
            type="text"
            on:input:target=move |v| {
                set_talent_str.set(v.target().value().parse());
            }
        />
        <TalentConfigView talent_config=talent_str />
        <TraitTreeDebug />
    }
}
