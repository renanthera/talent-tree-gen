use crate::talent_encoding::TalentEncoding;
use crate::version::{ProductType, Version};
use std::default::Default;

impl Default for ProductType {
    fn default() -> Self {
        ProductType::WOW
    }
}

impl Default for TalentEncoding {
    fn default() -> Self {
        TalentEncoding {
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

impl Default for Version {
    fn default() -> Self {
        Version {
            product: ProductType::default(),
            major: 11,
            patch: 2,
            minor: 0,
            build: 63003,
        }
    }
}
