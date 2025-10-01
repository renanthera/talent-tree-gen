use leptos::{either::Either, leptos_dom::logging::console_log, prelude::*};
use thaw::{Tooltip, TooltipAppearance};
use thiserror::Error;

use crate::talent_encoding::{TalentEncoding, TalentEncodingError};
use crate::trait_tree::{
    fetch_trait_trees, TraitTree, TraitTreeEntry, TraitTreeEntryType, TraitTreeNode,
    TraitTreeNodeType,
};

#[derive(Error, Debug, Clone, PartialEq)]
pub enum TalentConfigurationError {
    #[error(transparent)]
    TalentEncodingError(#[from] TalentEncodingError),
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
    // TODO: use hash_map instead of vec?
    pub selected_talents: Vec<TalentEntry>,
    pub unselected_talents: Vec<TalentEntry>,
    pub all_talents: Vec<TalentEntry>,
    pub subtrees: Vec<usize>,
    pub trait_tree: TraitTree,
}

impl TalentConfiguration {
    pub fn new_from_str(
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
            .clone()
            .into_iter()
            .chain(trait_tree.spec_nodes.clone().into_iter())
            .chain(trait_tree.hero_nodes.clone().into_iter())
            .chain(trait_tree.sub_tree_nodes.clone().into_iter())
            .collect::<Vec<_>>();
        talent_entries.sort_by(|a, b| (&a.id).cmp(&b.id));

        // TODO: encode number of allotted TTN in data
        let mut selected_talents: Vec<TalentEntry> = Vec::with_capacity(80);
        let mut unselected_talents: Vec<TalentEntry> = Vec::with_capacity(80);
        let mut all_talents: Vec<TalentEntry> = Vec::with_capacity(160);
        let mut subtrees: Vec<usize> = Vec::with_capacity(2);

        // TODO: when i have my own data format, no longer depend on node order vec
        // and get rid of all of this unwrap
        for entry in trait_tree.full_node_order.iter() {
            let mut skip: bool = false;
            let selected_node = match talent_entries.iter().find(|ttn| ttn.id == *entry) {
                Some(node) => node.clone(),
                None => {
                    skip = true;
                    TraitTreeNode::default()
                }
            };
            let mut selected_trait = match skip {
                true => TraitTreeEntry::default(),
                false => selected_node.entries.first().unwrap().clone(),
            };
            let mut rank: usize = 0;

            if get_bits(1) == 1 {
                // entry is selected
                rank = selected_node.max_ranks.unwrap_or(1);

                match selected_trait.node_type {
                    Some(TraitTreeEntryType::SubTree) => skip = true,
                    _ => (),
                };

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
                        selected_trait = selected_node.entries[choice_bits].clone();

                        match selected_trait.node_type {
                            Some(TraitTreeEntryType::SubTree) => {
                                subtrees.push(selected_trait.trait_sub_tree_id.expect(
                                    "A SubTree selection node does not have a trait_tree_id!",
                                ));
                            }
                            _ => (),
                        };
                    }
                }
                if !skip {
                    selected_talents.push(TalentEntry {
                        trait_tree_node: selected_node.clone(),
                        trait_tree_entry: selected_trait.clone(),
                        rank,
                    });
                }
            } else {
                if !skip {
                    unselected_talents.push(TalentEntry {
                        trait_tree_node: selected_node.clone(),
                        trait_tree_entry: selected_trait.clone(),
                        rank,
                    });
                }
            }
            all_talents.push(TalentEntry {
                trait_tree_node: selected_node,
                trait_tree_entry: selected_trait,
                rank,
            });
        }

        match config.is_valid(s, serialization_version) {
            Ok(()) => Ok(Self {
                string: s.to_string(),
                spec,
                selected_talents,
                unselected_talents,
                all_talents,
                subtrees,
                trait_tree,
            }),
            Err(err) => Err(TalentConfigurationError::TalentEncodingError(err)),
        }
    }

    pub fn new(
        config: TalentEncoding,
        trait_tree_data: Vec<TraitTree>,
    ) -> Result<Self, TalentConfigurationError> {
        let Some(trait_tree) = trait_tree_data
            .iter()
            .find(|trait_tree| trait_tree.spec_id == 268)
        else {
            panic!()
        };
        let unselected_talents = trait_tree
            .class_nodes
            .iter()
            .chain(trait_tree.spec_nodes.clone().iter())
            .chain(trait_tree.hero_nodes.clone().iter())
            .chain(trait_tree.sub_tree_nodes.clone().iter())
            .map(|node| TalentEntry {
                trait_tree_node: node.clone(),
                rank: 0,
                trait_tree_entry: Default::default(),
            })
            .collect::<Vec<_>>();

        Ok(Self {
            string: "".to_string(),
            spec: trait_tree.spec_id,
            all_talents: unselected_talents.clone(),
            unselected_talents,
            selected_talents: Default::default(),
            subtrees: Default::default(),
            trait_tree: trait_tree.clone(),
        })
    }

    fn compute_hero_talent_normalization(&self) -> (i32, i32) {
        // TODO: bake this into data
        let class_x_max = self
            .trait_tree
            .class_nodes
            .iter()
            .map(|entry| entry.pos_x)
            .max()
            .unwrap_or(0);

        let spec_x_min = self
            .trait_tree
            .spec_nodes
            .iter()
            .map(|entry| entry.pos_x)
            .min()
            .unwrap_or(100);

        let class_y_min = self
            .trait_tree
            .class_nodes
            .iter()
            .map(|entry| entry.pos_y)
            .min()
            .unwrap_or(0);

        let class_y_max = self
            .trait_tree
            .class_nodes
            .iter()
            .map(|entry| entry.pos_y)
            .max()
            .unwrap_or(100);

        let hero_y_min = self
            .trait_tree
            .hero_nodes
            .iter()
            .map(|entry| entry.pos_y)
            .min()
            .unwrap_or(0);

        let hero_y_max = self
            .trait_tree
            .hero_nodes
            .iter()
            .map(|entry| entry.pos_y)
            .max()
            .unwrap_or(100);

        let entry = self
            .trait_tree
            .hero_nodes
            .iter()
            .find(|entry| match entry.entry_node {
                Some(true) => self
                    .subtrees
                    .contains(&entry.trait_sub_tree_id.unwrap_or(0)),
                _ => false,
            })
            .unwrap();

        let (root_x, root_y) = (entry.pos_x, entry.pos_y);

        let (rv_x, rv_y) = (
            (class_x_max + spec_x_min) / 2 - root_x,
            (class_y_min + class_y_max) / 2 - (hero_y_max - hero_y_min) / 4 - root_y,
        );

        (rv_x, rv_y)
    }

    fn scale(&self, x: i32, y: i32) -> (i32, i32) {
        // TODO: bake this into data
        const SCALE_FACTOR: i32 = 15;

        (x / SCALE_FACTOR, y / SCALE_FACTOR)
    }

    fn coordinate_transformation(&self, entry: &TalentEntry) -> (i32, i32) {
        // TODO: bake this into data
        let (x_offset, y_offset) = match entry.trait_tree_node.trait_sub_tree_id {
            Some(_) => self.compute_hero_talent_normalization(),
            None => (0, 0),
        };

        match entry.trait_tree_node.trait_sub_tree_id {
            Some(_) => self.scale(
                entry.trait_tree_node.pos_x + x_offset,
                entry.trait_tree_node.pos_y + y_offset,
            ),
            None => self.scale(entry.trait_tree_node.pos_x, entry.trait_tree_node.pos_y),
        }
    }

    fn do_draw(&self, node: &TalentEntry) -> bool {
        if node.trait_tree_node.id == 0 {
            return false;
        }

        if node.trait_tree_node.node_type == TraitTreeNodeType::SubTree {
            return false;
        }

        match node.trait_tree_node.trait_sub_tree_id {
            Some(tst_id) => {
                if !self.subtrees.contains(&tst_id) {
                    return false;
                }
            }
            None => (),
        };

        true
    }

    fn draw_node(&self, node: &TalentEntry) -> impl IntoView {
        if !self.do_draw(node) {
            return Either::Right(view! {});
        }

        let (cx, cy) = self.coordinate_transformation(node);
        let name = match &node.trait_tree_entry.name {
            Some(n) => n.to_string(),
            None => node.trait_tree_node.name.to_string(),
        };
        let id = node.trait_tree_node.id.to_string();
        let color = match node.rank {
            0 => "red",
            _ => "green",
        };

        // TODO: unique ids even if multiple talent trees of the same spec are rendered
        Either::Left(view! {
            <Tooltip content=name appearance=TooltipAppearance::Normal>
                <circle cx=cx cy=cy r=10 fill=color id=id />
            </Tooltip>
        })
    }

    fn draw_nodes(&self) -> impl IntoView {
        self.all_talents
            .iter()
            .map(|entry| self.draw_node(entry))
            .collect::<Vec<_>>()
    }

    fn draw_line(&self, a: &TalentEntry, b: &TalentEntry) -> impl IntoView {
        if !self.do_draw(a) || !self.do_draw(b) {
            return Either::Right(view! {});
        }

        let (x_1, y_1) = self.coordinate_transformation(a);
        let (x_2, y_2) = self.coordinate_transformation(b);
        let color = match a.rank > 0 && b.rank > 0 {
            true => "green",
            false => "red",
        };

        Either::Left(view! { <line x1=x_1 y1=y_1 x2=x_2 y2=y_2 stroke=color /> })
    }

    fn draw_lines(&self) -> impl IntoView {
        self.all_talents
            .iter()
            .flat_map(|entry| {
                entry.trait_tree_node.next.iter().map(|b_id| {
                    let b = self
                        .all_talents
                        .iter()
                        .find(|te| te.trait_tree_node.id == *b_id)
                        .unwrap();
                    self.draw_line(entry, b)
                })
            })
            .collect::<Vec<_>>()
    }

    pub fn draw(&self) -> impl IntoView {
        view! {
            <svg view_box="0 0 1500 500" height=500 width=1500>
                {self.draw_lines()}
                {self.draw_nodes()}
            </svg>
        }
    }
}

#[component]
pub fn DrawTalentConfigView(
    talent_configuration: Memo<Result<TalentConfiguration, TalentConfigurationError>>,
) -> impl IntoView {
    let fallback = move |_| {
        view! { <div>{format!("{:?}", talent_configuration.get())}</div> }
    };

    view! {
        <div>
            <ErrorBoundary fallback>
                {move || {
                    talent_configuration
                        .with(|config| {
                            match config {
                                Ok(conf) => Either::Left(conf.draw()),
                                Err(e) => Either::Right(view! { {format!("{:?}", e)} }),
                            }
                        })
                }}
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
                            TalentConfiguration::new_from_str(
                                &talent_str.get(),
                                talent_encoding.get(),
                                trait_trees.clone(),
                            )
                        });
                        view! { <DrawTalentConfigView talent_configuration /> }
                    })
            })}
        </Transition>
    }
}
