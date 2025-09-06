use leptos::{leptos_dom::logging::console_log, prelude::*};
use std::str::FromStr;
use thiserror::Error;

use crate::talent_encoding::{TalentEncodingConfiguration, TalentEncodingError, Version};

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
    pub serialization_version: usize,
    pub spec: usize,
}

impl FromStr for TalentConfiguration {
    type Err = TalentConfigurationError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config = TalentEncodingConfiguration::new(Version::default())?;

        let mut bit_head: usize = 0;
        let mut iter = s.chars().peekable();

        let mut get_bits = |count: usize| -> Result<usize, TalentEncodingError> {
            let mut value: usize = 0;
            for offset in 0..count {
                let Some(char) = iter.peek() else {
                    return Ok(value);
                };
                console_log(&format!("{:?}", config.find_char(&char.to_string())));
                console_log(&format!("{:?}", bit_head % config.byte_size));
                value += (config.find_char(&char.to_string())? >> (bit_head % config.byte_size)
                    & 0b1)
                    << std::cmp::min(offset, 63);
                bit_head += 1;
                if bit_head % config.byte_size == 0 {
                    iter.next();
                }
            }
            Ok(value)
        };

        let serialization_version = get_bits(config.version_bits)?;
        let _ = get_bits(config.tree_bits)?;
        match config.is_valid(s, serialization_version) {
            Ok(()) => Ok(Self {
                string: s.to_string(),
                serialization_version,
                spec: get_bits(config.spec_bits)?,
            }),
            Err(err) => Err(TalentConfigurationError::TalentEncodingError(err)),
        }
    }
}

#[component]
pub fn TalentConfigView(
    talent_config: ReadSignal<Result<TalentConfiguration, TalentConfigurationError>>,
) -> impl IntoView {
    let fallback = move |error: ArcRwSignal<Errors>| {
        let error_list = move || {
            error.with(|error| {
                error
                    .iter()
                    .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                    .collect::<Vec<_>>()
            })
        };

        view! {
            <div class="error">
                <p>"Error"</p>
                <ul>{error_list}</ul>
            </div>
        }
    };
    view! {
        <ErrorBoundary fallback>
            <div>{move || format!("{0:?}", talent_config.get())}</div>
        </ErrorBoundary>
    }
}
