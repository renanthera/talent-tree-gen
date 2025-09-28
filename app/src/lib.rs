use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use thaw::ConfigProvider;

use crate::talent_configuration::{
    TalentConfigView, TalentConfiguration, TalentConfigurationError,
};
use crate::talent_encoding::TalentEncoding;
use crate::trait_tree::TraitTreeDebug;
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
    let (talent_encoding, set_talent_encoding) = signal(TalentEncoding::default());
    provide_context(set_talent_encoding);

    view! {
        <ConfigProvider>
            <div>{move || format!("{}", talent_encoding.get())}</div>
            <VersionView />
            <TalentConfigView talent_encoding />
            <TraitTreeDebug />
        </ConfigProvider>
    }
}
