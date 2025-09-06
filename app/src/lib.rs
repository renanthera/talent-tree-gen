use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};

use crate::talent_configuration::{
    TalentConfigView, TalentConfiguration, TalentConfigurationError,
};
use crate::talent_encoding::TalentEncodingConfiguration;

mod defaults;
mod talent_configuration;
mod talent_encoding;
mod trait_tree;

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

pub async fn fetch_versions() -> Result<Vec<TalentEncodingConfiguration>, Error> {
    Ok(reqwasm::http::Request::get("/versions.json")
        .send()
        .await?
        .json()
        .await?)
}

#[component]
fn HomePage() -> impl IntoView {
    let (talent_str, set_talent_str) = signal::<
        Result<TalentConfiguration, TalentConfigurationError>,
    >(Err(TalentConfigurationError::NoString));

    let (selected_talent_encoding, set_selected_talent_encoding) =
        signal(TalentEncodingConfiguration::default());

    let version_data = LocalResource::new(move || fetch_versions());

    view! {
        <input
            type="text"
            on:input:target=move |v| {
                set_talent_str.set(v.target().value().parse());
            }
        />
        <select name="version">
            <option value="default">
                {move || format!("{0}", selected_talent_encoding.get())}
            </option>
            <Transition fallback=|| {
                view! { <option>"Loading..."</option> }
            }>
                {move || Suspend::new(async move {
                    version_data
                        .await
                        .map(|tt_data| {
                            tt_data
                                .into_iter()
                                .map(|tt| {
                                    view! { <option>{move || format!("{tt}")}</option> }
                                })
                                .collect::<Vec<_>>()
                        })
                })}
            </Transition>
        </select>
        <TalentConfigView talent_config=talent_str />
    }
}
