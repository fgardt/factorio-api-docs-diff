use serde::{Deserialize, Serialize};
use structdiff::{Difference, StructDiff};

pub mod prototype;
pub mod runtime;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Difference, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Application {
    Factorio,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Difference, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Stage {
    Prototype,
    Runtime,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Difference, Clone)]
pub struct Common {
    pub application: Application,
    pub stage: Stage,
    pub application_version: String,
    pub api_version: u8,
}
