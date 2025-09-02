use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

mod talent_string;
mod trait_tree;

use crate::talent_string::TalentConfigView;
use crate::talent_string::TalentConfiguration;
use crate::talent_string::TalentParseError;
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

#[component]
fn HomePage() -> impl IntoView {
    let (talent_str, set_talent_str) =
        signal::<Result<TalentConfiguration, TalentParseError>>(Err(TalentParseError::NoString));

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
