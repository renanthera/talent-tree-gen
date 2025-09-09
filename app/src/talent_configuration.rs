use leptos::{leptos_dom::logging::console_log, prelude::*};
use thiserror::Error;

use crate::talent_encoding::{TalentEncoding, TalentEncodingError};
use crate::trait_tree::{
    fetch_trait_trees, TraitTree, TraitTreeEntry, TraitTreeEntryType, TraitTreeNode,
};

#[derive(Error, Debug, Clone)]
pub enum TalentConfigurationError {
    #[error(transparent)]
    TalentEncodingError(#[from] TalentEncodingError),
    #[error("No talent string")]
    NoString,
    #[error("Specialization not found in data.")]
    SpecNotFound,
}

#[derive(Debug, Clone)]
pub struct TalentEntry {
    pub rank: usize,
    pub trait_tree_entry: TraitTreeEntry,
}

#[derive(Debug, Clone)]
pub struct TalentConfiguration {
    pub string: String,
    pub spec: usize,
    pub talents_n: Vec<TalentEntry>,
    talent_store: Vec<TraitTreeNode>,
}

impl TalentConfiguration {
    fn new(
        s: &str,
        config: TalentEncoding,
        trait_tree_data: Vec<TraitTree>,
    ) -> Result<Self, TalentConfigurationError> {
        let mut bit_head: usize = 0;
        let mut iter = s.chars().peekable();

        let mut get_bits = |count: usize| -> usize {
            let mut value: usize = 0;
            for offset in 0..count {
                let Some(char) = iter.peek() else {
                    return value;
                };
                // already validated that char in config.base64_chars in config.is_valid, panic is fine
                let char_position = config.find_char_unchecked(&char.to_string());
                // use checked_shr to allow to shift into zero without panic
                let bit_index_set: usize = char_position
                    .checked_shr((bit_head % config.byte_size).try_into().unwrap())
                    .unwrap_or(0)
                    & 0b1;
                // use checked_shl to allow to shift into zero without panic
                // try_into should not panic unless `usize` is somehow a very small type
                value += bit_index_set
                    .checked_shl(std::cmp::min(offset, 63).try_into().unwrap())
                    .unwrap_or(0);
                bit_head += 1;
                if bit_head % config.byte_size == 0 {
                    iter.next();
                }
            }
            value
        };

        let serialization_version = get_bits(config.version_bits);
        let spec = get_bits(config.spec_bits);
        let _ = get_bits(config.tree_bits);

        let Some(trait_tree) = trait_tree_data.into_iter().find(|tt| tt.spec_id == spec) else {
            return Err(TalentConfigurationError::SpecNotFound);
        };

        let mut talent_entries = trait_tree
            .class_nodes
            .into_iter()
            .chain(trait_tree.spec_nodes.into_iter())
            .chain(trait_tree.hero_nodes.into_iter())
            .chain(trait_tree.sub_tree_nodes.into_iter())
            .collect::<Vec<_>>();
        talent_entries.sort_by(|a, b| (&a.id).cmp(&b.id));

        // TODO: encode number of allotted TTN in data
        let mut talents: Vec<TalentEntry> = Vec::with_capacity(80);

        // for entry in trait_tree.full_node_order {
        for selected_node in talent_entries.iter() {
            console_log(&format!("{:?}", selected_node.name));
            // if get_bits(1) == 1 {
            //     // entry is selected
            //     console_log(&format!("{:?}", selected_node));
            //     let mut selected_trait = selected_node.entries.first().unwrap();
            //     let mut rank: usize = selected_node.max_ranks.unwrap_or(1);
            //     let mut skip: bool = false;

            //     if selected_trait.clone().node_type.unwrap() == TraitTreeEntryType::SubTree {
            //         skip = true;
            //     }

            //     // entry is purchased, no choice or rank bits
            //     // otherwise,
            //     if get_bits(1) == 1 {
            //         if get_bits(1) == 1 {
            //             // partially ranked
            //             rank = get_bits(config.rank_bits);
            //         }

            //         if get_bits(1) == 1 {
            //             // choice
            //             let choice_bits = get_bits(config.choice_bits);
            //             console_log(&format!("{:?}", choice_bits));
            //             console_log(&format!("{:?}", selected_node.entries));
            //             selected_trait = &selected_node.entries[choice_bits];
            //         }
            //     }
            //     if !skip {
            //         talents.push(TalentEntry {
            //             trait_tree_entry: selected_trait.clone(),
            //             rank
            //         });
            //     }
            // }
        }

        match config.is_valid(s, serialization_version) {
            Ok(()) => Ok(Self {
                string: s.to_string(),
                spec,
                talents_n: talents,
                talent_store: talent_entries,
            }),
            Err(err) => Err(TalentConfigurationError::TalentEncodingError(err)),
        }
    }
}

#[component]
pub fn TalentConfigView(talent_encoding: ReadSignal<TalentEncoding>) -> impl IntoView {
    let (talent_str, set_talent_str) = signal("".to_string());

    let talent_configuration = move |trait_tree_data: Vec<TraitTree>| {
        TalentConfiguration::new(&talent_str.get(), talent_encoding.get(), trait_tree_data)
    };

    let trait_tree_data = LocalResource::new(move || fetch_trait_trees());

    let fallback = || view! { <div>"Loading..."</div> };

    view! {
        <input type="text" on:input:target=move |tag| set_talent_str.set(tag.target().value()) />
        <Transition fallback>
            {move || Suspend::new(async move {
                trait_tree_data
                    .await
                    .map(|trait_trees| {
                        view! {
                            <div>
                                {move || {
                                    format!("{0:?}", talent_configuration(trait_trees.clone()))
                                }}
                            </div>
                        }
                    })
            })}
        </Transition>
    }
}
