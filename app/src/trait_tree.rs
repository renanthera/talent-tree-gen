use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TraitTree {
    pub trait_tree_id: i32,
    pub class_id: i32,
    pub spec_id: i32,
    pub class_name: String,
    pub spec_name: String,
    pub class_nodes: Vec<TraitTreeNode>,
    pub spec_nodes: Vec<TraitTreeNode>,
    pub hero_nodes: Vec<TraitTreeNode>,
    pub sub_tree_nodes: Vec<TraitTreeNode>,
    pub full_node_order: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TraitTreeNode {
    pub id: i32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub max_ranks: Option<i32>,
    pub name: String,
    #[serde(
        rename = "type",
        deserialize_with = "trait_tree_node_type_deserializer::deserialize"
    )]
    pub node_type: TraitTreeNodeType,
    pub entry_node: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraitTreeNodeType {
    Single,
    Choice,
    SubTree,
}

mod trait_tree_node_type_deserializer {
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
