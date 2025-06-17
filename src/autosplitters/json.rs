use std::path::Path;

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Check {
    pub name: String,
    #[serde(flatten)]
    pub check: CheckDetails,
    // Like a logical and of checks
    pub more: Option<Vec<CheckDetails>>,
    // A sequence of things to check similar to a logic and, but allows fewer reads of memory
    pub next: Option<Vec<CheckDetails>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckType {
    Bit,
    Eq,
    Gt,
    Lt,
    Gte,
    Lte,
    Wbit,
    Weq,
    Wgt,
    Wlt,
    Wgte,
    Wlte,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckDetails {
    pub note: Option<String>,
    #[serde(deserialize_with = "from_hex")]
    pub address: u32,
    #[serde(deserialize_with = "from_hex")]
    pub value: u32,
    #[serde(rename = "type")]
    pub typ: CheckType,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Splits {
    pub game: String,
    pub autostart: Option<Autostart>,
    pub definitions: Vec<Check>,
}

impl Splits {
    pub fn parse(input: &str) -> Result<Splits, Box<dyn std::error::Error>> {
        serde_json::from_str(input).map_err(|e| e.into())
    }

    pub fn from_file<P>(p: P) -> Result<Splits, Box<dyn std::error::Error>>
    where
        P: AsRef<Path>,
    {
        let s = std::fs::read_to_string(p)?;
        Self::parse(&s)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Autostart {
    #[serde(deserialize_with = "string_to_bool")]
    pub active: bool,
    #[serde(flatten)]
    pub check: CheckDetails,
}

fn from_hex<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    u32::from_str_radix(s.trim().trim_start_matches("0x"), 16)
        .map_err(|e| de::Error::custom(format!("invalid hex: {}", e)))
}

fn string_to_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    match s.trim() {
        "1" => Ok(true),
        "0" => Ok(false),
        _ => Err(de::Error::custom(format!("invalid boolean string: {}", s))),
    }
}
