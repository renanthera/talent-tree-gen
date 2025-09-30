use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

use crate::version::Version;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum TalentEncodingError {
    #[error("Talent string contains characters not permitted for encoding configuration")]
    InvalidBase64Charset,
    #[error("String too short")]
    StringTooShort,
    #[error("Serialization version does not match encoding configuration")]
    IncorrectSerializationVersion,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TalentEncoding {
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

impl fmt::Display for TalentEncoding {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.version.fmt(f)
    }
}

impl TalentEncoding {
    // pub fn find_char(&self, c: &str) -> Result<usize, TalentEncodingError> {
    //     self.base64_chars
    //         .find(c)
    //         .ok_or(TalentEncodingError::InvalidBase64Charset)
    // }

    pub fn find_char_unchecked(&self, c: &str) -> usize {
        self.base64_chars.find(c).unwrap()
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
