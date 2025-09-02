use leptos::{leptos_dom::logging::console_log, prelude::*};
use regex::Regex;
use std::fmt::Display;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum TalentParseError {
    #[error(transparent)]
    TalentEncodingError(#[from] TalentEncodingError),
    #[error("Talent string is not valid base64")]
    InvalidBase64,
    #[error("No talent string")]
    NoString,
}

#[derive(Error, Debug, Clone)]
pub enum TalentEncodingError {
    #[error("Version not found in database")]
    VersionNotFound,
}

#[derive(Debug, Clone)]
pub struct Talent {
    id: i32,
}

#[derive(Debug, Clone)]
pub struct TalentConfiguration {
    pub str: String,
    talents: Vec<Talent>,
}

#[derive(Debug)]
struct Version {
    major: i32,
    patch: i32,
    minor: i32,
}

#[derive(Debug)]
struct TalentEncodingConfiguration {
    version: Version,
    base64_chars: String,
    serialization_version: i32,
    version_bits: usize,
    spec_bits: usize,
    tree_bits: usize,
    rank_bits: usize,
    choice_bits: usize,
    byte_size: usize,
}

impl TalentEncodingConfiguration {
    pub fn new(version: Version) -> Result<Self, TalentEncodingError> {
        match version {
            Version {
                major: 11,
                patch: 2,
                minor: 5,
            } => Ok(TalentEncodingConfiguration::default()),
            _ => Err(TalentEncodingError::VersionNotFound),
        }
    }

    fn escaped_chars(&self) -> String {
        let mut rv = self.base64_chars.clone();
        let unescaped_slash = Regex::new(r"[^\\]/").unwrap();
        while let Some(m) = unescaped_slash.find(rv.as_str()) {
            rv.insert(m.start() + 1, '\\');
            console_log(&rv);
        }
        rv
    }

    pub fn valid_base64(&self, string: &str) -> bool {
        let match_str = format!(r"[^{}]", self.escaped_chars());
        let re = Regex::new(match_str.as_str()).unwrap();
        match re.find(string) {
            Some(_) => false,
            None => true,
        }
    }
}

impl Default for TalentConfiguration {
    fn default() -> Self {
        TalentConfiguration {
            str: "".to_string(),
            talents: Vec::new(),
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Version {
            major: 11,
            patch: 2,
            minor: 5,
        }
    }
}

impl Default for TalentEncodingConfiguration {
    fn default() -> Self {
        TalentEncodingConfiguration {
            version: Version::default(),
            base64_chars: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/"
                .to_string(),
            serialization_version: 2,
            version_bits: 8,
            spec_bits: 16,
            tree_bits: 128,
            rank_bits: 6,
            choice_bits: 2,
            byte_size: 6,
        }
    }
}

impl FromStr for TalentConfiguration {
    type Err = TalentParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let config = TalentEncodingConfiguration::new(Version {
            major: 11,
            patch: 2,
            minor: 5,
        })?;

        if !config.valid_base64(s) {
            return Err(TalentParseError::InvalidBase64);
        }

        Ok(Self {
            str: s.to_string(),
            talents: Vec::new(),
        })
    }
}

impl Display for TalentConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "TalentConfiguration {{ str: {} }}", self.str)
    }
}

#[component]
pub fn TalentConfigView(
    talent_config: ReadSignal<Result<TalentConfiguration, TalentParseError>>,
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
