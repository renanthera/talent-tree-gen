use crate::trait_tree::trait_tree_node_type_deserializer;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum TalentParseError {
    #[error(transparent)]
    TalentEncodingError(#[from] TalentEncodingError),
    #[error("No talent string")]
    NoString,
}

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

#[derive(Debug, Clone)]
pub struct TalentConfiguration {
    pub string: String,
    pub serialization_version: usize,
    pub spec: usize,
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

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TraitTree {
    pub trait_tree_id: i32,
    pub class_id: i32,
    pub spec_id: i32,
    pub class_name: String,
    pub spec_name: String,
    pub class_nodes: Vec<TraitTreeNode>,
    pub spec_nodes: Vec<TraitTreeNode>,
    pub hero_nodes: Vec<TraitTreeNode>,
    pub sub_tree_nodes: Vec<TraitTreeNode>,
    pub full_node_order: Vec<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "camelCase"))]
pub struct TraitTreeNode {
    pub id: i32,
    pub pos_x: i32,
    pub pos_y: i32,
    pub max_ranks: Option<i32>,
    pub name: String,
    #[serde(
        rename = "type",
        deserialize_with = "trait_tree_node_type_deserializer::deserialize"
    )]
    pub node_type: TraitTreeNodeType,
    pub entry_node: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraitTreeNodeType {
    Single,
    Choice,
    SubTree,
}
