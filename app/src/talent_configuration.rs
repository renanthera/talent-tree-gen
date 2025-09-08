use leptos::{leptos_dom::logging::console_log, prelude::*};
use thiserror::Error;

use crate::talent_encoding::{TalentEncoding, TalentEncodingError};
use crate::trait_tree::{fetch_trait_trees, TraitTree};

#[derive(Error, Debug, Clone)]
pub enum TalentConfigurationError {
    #[error(transparent)]
    TalentEncodingError(#[from] TalentEncodingError),
    #[error("No talent string")]
    NoString,
}

#[derive(Debug, Clone)]
pub struct TalentConfiguration {
    pub string: String,
    pub spec: usize,
}

impl TalentConfiguration {
    fn new(
        s: &str,
        config: &TalentEncoding,
        trait_tree_data: &Vec<TraitTree>,
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
        match config.is_valid(s, serialization_version) {
            Ok(()) => Ok(Self {
                string: s.to_string(),
                spec,
            }),
            Err(err) => Err(TalentConfigurationError::TalentEncodingError(err)),
        }
    }
}

#[component]
pub fn TalentConfigView(talent_encoding: ReadSignal<TalentEncoding>) -> impl IntoView {
    let (talent_str, set_talent_str) = signal("".to_string());

    let talent_configuration = move |trait_tree_data: &Vec<TraitTree>| {
        TalentConfiguration::new(&talent_str.get(), &talent_encoding.get(), trait_tree_data)
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
                                {move || format!("{0:?}", talent_configuration(&trait_trees))}
                            </div>
                        }
                    })
            })}
        </Transition>
    }
}
