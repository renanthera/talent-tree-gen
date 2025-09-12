use leptos::{leptos_dom::logging::console_log, prelude::*};
use thiserror::Error;

use crate::talent_encoding::{TalentEncoding, TalentEncodingError};
use crate::trait_tree::{
    fetch_trait_trees, TraitTree, TraitTreeEntry, TraitTreeEntryType, TraitTreeNode,
};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum TalentConfigurationError {
    #[error(transparent)]
    TalentEncodingError(#[from] TalentEncodingError),
    #[error("No talent string")]
    NoString,
    #[error("Specialization not found in data.")]
    SpecNotFound,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TalentEntry {
    pub rank: usize,
    pub trait_tree_node: TraitTreeNode,
    pub trait_tree_entry: TraitTreeEntry,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TalentConfiguration {
    pub string: String,
    pub spec: usize,
    pub talents_n: Vec<TalentEntry>,
    pub talent_store: Vec<TraitTreeNode>,
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

        // TODO: when i have my own data format, no longer depend on node order vec
        // and get rid of all of this unwrap
        for entry in trait_tree.full_node_order {
            if get_bits(1) == 1 {
                // entry is selected
                let selected_node = talent_entries.iter().find(|ttn| ttn.id == entry).unwrap();
                // console_log(&format!("{:?}", selected_node));
                let mut selected_trait = selected_node.entries.first().unwrap();
                let mut rank: usize = selected_node.max_ranks.unwrap_or(1);
                let mut skip: bool = false;

                if selected_trait.node_type == Some(TraitTreeEntryType::SubTree) {
                    skip = true;
                }

                // entry is purchased, no choice or rank bits
                // otherwise,
                if get_bits(1) == 1 {
                    if get_bits(1) == 1 {
                        // partially ranked
                        rank = get_bits(config.rank_bits);
                    }

                    if get_bits(1) == 1 {
                        // choice
                        let choice_bits = get_bits(config.choice_bits);
                        console_log(&format!("{:?}", choice_bits));
                        console_log(&format!("{:?}", selected_node.entries));
                        selected_trait = &selected_node.entries[choice_bits];
                    }
                }
                if !skip {
                    talents.push(TalentEntry {
                        trait_tree_entry: selected_trait.clone(),
                        trait_tree_node: selected_node.clone(),
                        rank,
                    });
                }
            }
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
pub fn DrawTalentConfigView(
    configuration: Memo<Result<TalentConfiguration, TalentConfigurationError>>,
) -> impl IntoView {
    let draw_node = |node: TraitTreeNode, color: &str| {
        let cx = node.pos_x / 25;
        let cy = node.pos_y / 25;
        view! { <circle cx=cx cy=cy r=5 fill=color /> }.into_any()
    };
    view! {
        <div>
            <ErrorBoundary fallback=move |_| {
                view! { <div>{format!("{:?}", configuration.get())}</div> }
            }>
                <svg view_box="0 0 1000 500" height=500 width=1000>
                    {move || match configuration.get() {
                        Ok(config) => {
                            let mut u = config
                                .talent_store
                                .into_iter()
                                .map(|talent_entry| draw_node(talent_entry, "red"))
                                .collect::<Vec<_>>();
                            let mut v = config
                                .talents_n
                                .into_iter()
                                .map(|talent_entry| draw_node(
                                    talent_entry.trait_tree_node,
                                    "green",
                                ))
                                .collect::<Vec<_>>();
                            u.extend(v);
                            u
                        }
                        Err(_) => vec![view! {}.into_any()],
                    }}
                </svg>
            </ErrorBoundary>
        </div>
    }
}

#[component]
pub fn TalentConfigView(talent_encoding: ReadSignal<TalentEncoding>) -> impl IntoView {
    let (talent_str, set_talent_str) = signal("CwQAAAAAAAAAAAAAAAAAAAAAAAAAAgZZzYGzYWmx2YmZMAAAAAAAWAxMDmhZsYGsNzMjZMMzsMLm22sNbzMD2AAgNEAAAz2s0MzMLMYD".to_string());

    let trait_tree_data = LocalResource::new(move || fetch_trait_trees());

    let fallback = || view! { <div>"Loading..."</div> };

    view! {
        <input type="text" on:input:target=move |tag| set_talent_str.set(tag.target().value()) />
        <Transition fallback>
            {move || Suspend::new(async move {
                trait_tree_data
                    .await
                    .map(|trait_trees| {
                        let talent_configuration = Memo::new(move |_| {
                            TalentConfiguration::new(
                                &talent_str.get(),
                                talent_encoding.get(),
                                trait_trees.clone(),
                            )
                        });
                        view! {
                            <DrawTalentConfigView configuration=talent_configuration />
                            <div>{move || { format!("{0:?}", talent_configuration.get()) }}</div>
                        }
                    })
            })}
        </Transition>
    }
}
