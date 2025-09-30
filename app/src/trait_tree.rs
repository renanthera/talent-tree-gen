use leptos::prelude::*;
use serde::{Deserialize, Serialize};
/*
 * Custom Trait Tree Data Format:

pre-scale positions
pre-normalize hero talent positions
pre-calculate line segments end points?

struct TraitTree {
  class_id?,
  spec_id,
  class_name,
  spec_name,
  nodes, // ordered by node_order
}

struct TraitNode {
  id?,
  spell_id?,
  name,
  max_ranks?, # just put on entry
  node_type, # single, choice, subtree
  node_source, # class, spec, hero
  trait_sub_tree_id,
  position, # x,y coords
  entry_node, # start of tree
  free_node, # free allocation
  node_tier, # row info
  next, # nodes below
  entries, # trait_entries
}

struct TraitEntry {
  id,
  definition_id?,
  index?,
  spell_id?,
  max_ranks,
  name,
}

 */

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TraitTree {
    pub trait_tree_id: usize,
    pub class_id: usize,
    pub spec_id: usize,
    pub class_name: String,
    pub spec_name: String,
    pub class_nodes: Vec<TraitTreeNode>,
    pub spec_nodes: Vec<TraitTreeNode>,
    pub hero_nodes: Vec<TraitTreeNode>,
    pub sub_tree_nodes: Vec<TraitTreeNode>,
    pub full_node_order: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TraitTreeEntry {
    pub id: Option<usize>,
    pub definition_id: Option<usize>,
    pub max_ranks: Option<usize>,
    #[serde(
        rename = "type",
        default,
        deserialize_with = "trait_tree_entry_type_deserializer::deserialize"
    )]
    pub node_type: Option<TraitTreeEntryType>,
    pub name: Option<String>,
    pub spell_id: Option<usize>,
    pub index: Option<usize>,
    pub trait_sub_tree_id: Option<usize>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TraitTreeNode {
    pub id: usize,
    pub pos_x: i32,
    pub pos_y: i32,
    pub max_ranks: Option<usize>,
    pub name: String,
    #[serde(
        rename = "type",
        deserialize_with = "trait_tree_node_type_deserializer::deserialize"
    )]
    pub node_type: TraitTreeNodeType,
    pub entry_node: Option<bool>,
    pub next: Vec<usize>,
    pub prev: Vec<usize>,
    pub entries: Vec<TraitTreeEntry>,
    #[serde(alias = "subTreeId")]
    pub trait_sub_tree_id: Option<usize>,
}

impl Default for TraitTreeEntry {
    fn default() -> Self {
        Self {
            id: None,
            definition_id: None,
            max_ranks: None,
            node_type: None,
            name: None,
            spell_id: None,
            index: None,
            trait_sub_tree_id: None,
        }
    }
}

impl Default for TraitTreeNode {
    fn default() -> Self {
        Self {
            id: 0,
            pos_x: 0,
            pos_y: 0,
            max_ranks: None,
            name: "".to_string(),
            node_type: TraitTreeNodeType::Single,
            entry_node: None,
            next: Default::default(),
            prev: Default::default(),
            entries: Default::default(),
            trait_sub_tree_id: Default::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraitTreeEntryType {
    Active,
    Passive,
    SubTree,
}

mod trait_tree_entry_type_deserializer {
    use super::TraitTreeEntryType;
    use serde::de::{Error, Unexpected};
    // use leptos::leptos_dom::logging::console_log;

    pub fn deserialize<'de, D>(de: D) -> Result<Option<TraitTreeEntryType>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(de)?;
        // console_log(s);
        match s {
            "active" => Ok(Some(TraitTreeEntryType::Active)),
            "passive" => Ok(Some(TraitTreeEntryType::Passive)),
            "subtree" => Ok(Some(TraitTreeEntryType::SubTree)),
            "" => Ok(None),
            _ => Err(D::Error::invalid_value(
                Unexpected::Str(s),
                &"active/passive",
            )),
        }
    }
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
                                    match tt.spec_id {
                                        268 => view! { <li>{format!("{tt:?}")}</li> }.into_any(),
                                        _ => view! {}.into_any(),
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                })}
            </ul>
        </Transition>
    }
}
