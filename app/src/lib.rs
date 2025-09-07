use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use crate::talent_configuration::{
    TalentConfigView, TalentConfiguration, TalentConfigurationError,
};
use crate::version::VersionView;

mod defaults;
mod talent_configuration;
mod talent_encoding;
mod trait_tree;
mod version;

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
    let (talent_str, set_talent_str) = signal::<
        Result<TalentConfiguration, TalentConfigurationError>,
    >(Err(TalentConfigurationError::NoString));

    view! {
        <input
            type="text"
            on:input:target=move |v| {
                set_talent_str.set(v.target().value().parse());
            }
        />
        <VersionView />
        <TalentConfigView talent_config=talent_str />
    }
}
