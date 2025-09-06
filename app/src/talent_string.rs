use crate::trait_types::{
    ProductType, TalentConfiguration, TalentEncodingConfiguration, TalentEncodingError,
    TalentParseError, Version,
};
use leptos::prelude::*;
use regex::Regex;
use std::str::FromStr;
use leptos::leptos_dom::logging::console_log;

impl TalentEncodingConfiguration {
    fn new(version: Version) -> Result<Self, TalentEncodingError> {
        match version {
            _ if version == Version::default() => Ok(TalentEncodingConfiguration::default()),
            _ => Err(TalentEncodingError::VersionNotFound),
        }
    }

    fn find_char(&self, c: &str) -> Result<usize, TalentEncodingError> {
        self.base64_chars
            .find(c)
            .ok_or(TalentEncodingError::InvalidBase64Charset)
    }

    fn escaped_chars(&self) -> String {
        let mut rv = self.base64_chars.clone();
        let unescaped_slash = Regex::new(r"[^\\]/").unwrap();
        while let Some(m) = unescaped_slash.find(rv.as_str()) {
            rv.insert(m.start() + 1, '\\');
        }
        rv
    }

    fn valid_base64(&self, string: &str) -> Result<(), TalentEncodingError> {
        let match_str = format!(r"[^{}]+", self.escaped_chars());
        let re = Regex::new(match_str.as_str()).unwrap();
        match re.find(string) {
            Some(_) => Err(TalentEncodingError::InvalidBase64Charset),
            None => Ok(()),
        }
    }

    fn valid_size(&self, string: &str) -> Result<(), TalentEncodingError> {
        match self.version_bits + self.spec_bits + self.tree_bits <= string.len() * self.byte_size {
            true => Ok(()),
            false => Err(TalentEncodingError::StringTooShort),
        }
    }

    fn valid_version(&self, version: usize) -> Result<(), TalentEncodingError> {
        match self.serialization_version == version {
            true => Ok(()),
            false => Err(TalentEncodingError::IncorrectSerializationVersion),
        }
    }

    fn is_valid(&self, string: &str, version: usize) -> Result<(), TalentEncodingError> {
        self.valid_base64(string)?;
        self.valid_size(string)?;
        self.valid_version(version)?;

        Ok(())
    }
}

impl std::fmt::Display for ProductType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ProductType::WOW => write!(f, "Live"),
            ProductType::WOW_BETA => write!(f, "Beta"),
            ProductType::WOWDEV => write!(f, "Alpha"),
            ProductType::WOWT => write!(f, "PTR"),
            ProductType::WOWXPTR => write!(f, "XPTR"),
        }
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {}.{}.{}-{}",
            self.product, self.major, self.patch, self.minor, self.build
        )
    }
}

impl std::fmt::Display for TalentEncodingConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.version.fmt(f)
    }
}

impl FromStr for TalentConfiguration {
    type Err = TalentParseError;
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
                console_log(&format!("{:?}",config.find_char(&char.to_string())));
                console_log(&format!("{:?}",bit_head%config.byte_size));
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
            Err(err) => Err(TalentParseError::TalentEncodingError(err)),
        }
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
