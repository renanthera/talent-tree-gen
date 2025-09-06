use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum TalentEncodingError {
    #[error("Version not found in database")]
    VersionNotFound,
    #[error("Talent string contains characters not permitted for encoding configuration")]
    InvalidBase64Charset,
    #[error("String too short")]
    StringTooShort,
    #[error("Serialization version does not match encoding configuration")]
    IncorrectSerializationVersion,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductType {
    WOW,
    #[allow(non_camel_case_types)]
    WOW_BETA,
    WOWDEV,
    WOWT,
    WOWXPTR,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Version {
    pub product: ProductType,
    pub major: usize,
    pub patch: usize,
    pub minor: usize,
    pub build: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TalentEncodingConfiguration {
    pub version: Version,
    pub base64_chars: String,
    pub serialization_version: usize,
    pub version_bits: usize,
    pub spec_bits: usize,
    pub tree_bits: usize,
    pub rank_bits: usize,
    pub choice_bits: usize,
    pub byte_size: usize,
}

impl fmt::Display for ProductType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProductType::WOW => write!(f, "Live"),
            ProductType::WOW_BETA => write!(f, "Beta"),
            ProductType::WOWDEV => write!(f, "Alpha"),
            ProductType::WOWT => write!(f, "PTR"),
            ProductType::WOWXPTR => write!(f, "XPTR"),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{} {}.{}.{}-{}",
            self.product, self.major, self.patch, self.minor, self.build
        )
    }
}

impl fmt::Display for TalentEncodingConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.version.fmt(f)
    }
}

impl TalentEncodingConfiguration {
    pub fn new(version: Version) -> Result<Self, TalentEncodingError> {
        match version {
            _ if version == Version::default() => Ok(TalentEncodingConfiguration::default()),
            _ => Err(TalentEncodingError::VersionNotFound),
        }
    }

    pub fn find_char(&self, c: &str) -> Result<usize, TalentEncodingError> {
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

    pub fn is_valid(&self, string: &str, version: usize) -> Result<(), TalentEncodingError> {
        self.valid_base64(string)?;
        self.valid_size(string)?;
        self.valid_version(version)?;

        Ok(())
    }
}
