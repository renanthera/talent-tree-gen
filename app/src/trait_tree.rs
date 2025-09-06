use crate::trait_types::{TraitTree, TraitTreeNodeType};
use leptos::prelude::*;

pub mod trait_tree_node_type_deserializer {
    use super::TraitTreeNodeType;
    use serde::de::{Error, Unexpected};

    pub fn deserialize<'de, D>(de: D) -> Result<TraitTreeNodeType, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(de)?;
        match s {
            "single" => Ok(TraitTreeNodeType::Single),
            "choice" => Ok(TraitTreeNodeType::Choice),
            "subtree" => Ok(TraitTreeNodeType::SubTree),
            _ => Err(D::Error::invalid_value(
                Unexpected::Str(s),
                &"single/choice/subtree",
            )),
        }
    }
}

pub async fn fetch_trait_trees() -> Result<Vec<TraitTree>, Error> {
    Ok(reqwasm::http::Request::get("/talent-data/talents.json")
        .send()
        .await?
        .json()
        .await?)
}

#[component]
pub fn TraitTreeDebug() -> impl IntoView {
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
                                    view! { <li>{format!("{tt:?}")}</li> }
                                })
                                .collect::<Vec<_>>()
                        })
                })}
            </ul>
        </Transition>
    }
}
