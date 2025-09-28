use leptos::{html, leptos_dom::logging::console_log, prelude::*, svg};
use thaw::ConfigProvider;
use thaw::{Tooltip, TooltipAppearance};
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
    pub selected_talents: Vec<TalentEntry>,
    pub unselected_talents: Vec<TalentEntry>,
    pub subtrees: Vec<usize>,
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
                // try_into should not panic unless `usize` is somehow a very small type on target platform
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
        let mut selected_talents: Vec<TalentEntry> = Vec::with_capacity(80);
        let mut unselected_talents: Vec<TalentEntry> = Vec::with_capacity(80);
        let mut subtrees: Vec<usize> = Vec::with_capacity(2);

        // TODO: when i have my own data format, no longer depend on node order vec
        // and get rid of all of this unwrap
        let mut skip: bool = false;
        for entry in trait_tree.full_node_order {
            let selected_node_option = talent_entries.iter().find(|ttn| ttn.id == entry);
            if selected_node_option.is_none() {
                // console_log(&format!("{entry}"));
                skip = true;
            }
            // console_log(&format!("{:?}", selected_node_option));
            let selected_node = selected_node_option
                .cloned()
                .unwrap_or(TraitTreeNode::default());
            if get_bits(1) == 1 {
                // entry is selected
                // console_log(&format!("{}-{}", selected_node.id, selected_node.name));
                let mut selected_trait: &TraitTreeEntry = &Default::default();
                if skip == false {
                    selected_trait = selected_node.entries.first().unwrap();
                }
                let mut rank: usize = selected_node.max_ranks.unwrap_or(1);

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
                        selected_trait = &selected_node.entries[choice_bits];

                        if selected_trait.node_type == Some(TraitTreeEntryType::SubTree) {
                            // console_log(&format!("{:?}\n{:?}", selected_node, selected_trait));
                            subtrees.push(
                                selected_trait.trait_sub_tree_id.expect(
                                    "A SubTree selection node does not have a trait_tree_id!",
                                ),
                            );
                        }
                    }
                }
                if !skip {
                    selected_talents.push(TalentEntry {
                        trait_tree_entry: selected_trait.clone(),
                        trait_tree_node: selected_node.clone(),
                        rank,
                    });
                }
            } else {
                if !skip {
                    unselected_talents.push(TalentEntry {
                        trait_tree_node: selected_node.clone(),
                        trait_tree_entry: Default::default(),
                        rank: 0,
                    });
                }
            }
            skip = false;
        }

        console_log(&format!("{:?}", subtrees));

        match config.is_valid(s, serialization_version) {
            Ok(()) => Ok(Self {
                string: s.to_string(),
                spec,
                selected_talents,
                unselected_talents,
                subtrees,
            }),
            Err(err) => Err(TalentConfigurationError::TalentEncodingError(err)),
        }
    }
}

#[component]
pub fn DrawTalentConfigView(
    configuration: Memo<Result<TalentConfiguration, TalentConfigurationError>>,
) -> impl IntoView {
    let draw_nodes = |subtrees: &Vec<usize>, nodes: Vec<TalentEntry>, color: &'static str| {
        let draw_node = |node: &TalentEntry| {
            let cx = node.trait_tree_node.pos_x / 15;
            let cy = node.trait_tree_node.pos_y / 15;
            let name = match &node.trait_tree_entry.name {
                Some(n) => n.to_string(),
                None => node.trait_tree_node.name.to_string(),
            };
            let id = node.trait_tree_node.id.to_string();

            view! {
                <Tooltip content=name appearance=TooltipAppearance::Normal>
                    <circle cx=cx cy=cy r=10 fill=color title=id />
                </Tooltip>
            }
            .into_any()
        };
        nodes
            .iter()
            .map(|entry| match entry.trait_tree_node.trait_sub_tree_id {
                Some(tst_id) => match subtrees.contains(&tst_id) {
                    true => draw_node(entry),
                    false => view! {}.into_any(),
                },
                None => draw_node(entry),
            })
            .collect::<Vec<_>>()
    };
    view! {
        <div>
            <ErrorBoundary fallback=move |_| {
                view! { <div>{format!("{:?}", configuration.get())}</div> }
            }>
                <svg view_box="0 0 1500 500" height=500 width=1500>
                    {move || match configuration.get() {
                        Ok(config) => {
                            let mut u = draw_nodes(
                                &config.subtrees,
                                config.unselected_talents,
                                "red",
                            );
                            let mut v = draw_nodes(
                                &config.subtrees,
                                config.selected_talents,
                                "green",
                            );
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
        <input
            type="text"
            on:input:target=move |tag| set_talent_str.set(tag.target().value())
            value=talent_str
        />
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
