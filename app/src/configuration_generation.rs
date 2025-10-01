use leptos::prelude::*;

use crate::talent_configuration::{DrawTalentConfigView, TalentConfiguration};
use crate::trait_tree::fetch_trait_trees;
use crate::TalentEncoding;

#[component]
pub fn TalentConfigurationGeneration(talent_encoding: ReadSignal<TalentEncoding>) -> impl IntoView {
    let trait_tree_data = LocalResource::new(move || fetch_trait_trees());

    let fallback = || view! { <div>"Loading..."</div> };

    view! {
        <Transition fallback>
            {move || Suspend::new(async move {
                trait_tree_data
                    .await
                    .map(|trait_trees| {
                        let talent_configuration = Memo::new(move |_| {
                            TalentConfiguration::new(talent_encoding.get(), trait_trees.clone())
                        });
                        view! { <DrawTalentConfigView talent_configuration /> }
                    })
            })}
        </Transition>
    }
}
