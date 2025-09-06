use crate::trait_types::{TalentEncodingConfiguration, Version};
use std::default::Default;

impl Default for Version {
    fn default() -> Self {
        Version {
            major: 11,
            patch: 2,
            minor: 0,
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
