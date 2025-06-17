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
    #[serde(alias = "name", alias = "game")]
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

#[derive(Debug, Clone, Serialize)]
pub enum Autostart {
    Inactive { note: Option<String> },
    Active { check: CheckDetails },
}

impl<'de> Deserialize<'de> for Autostart {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = RawAutostart::deserialize(deserializer)?;

        if raw.active {
            let address = raw
                .address
                .ok_or_else(|| de::Error::missing_field("address"))?;
            let value = raw.value.ok_or_else(|| de::Error::missing_field("value"))?;
            let typ = raw.typ.ok_or_else(|| de::Error::missing_field("type"))?;

            let address = from_hex_str::<D>(&address)?;
            let value = from_hex_str::<D>(&value)?;
            Ok(Autostart::Active {
                check: CheckDetails {
                    note: raw.note,
                    address,
                    value,
                    typ,
                },
            })
        } else {
            Ok(Autostart::Inactive { note: raw.note })
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RawAutostart {
    #[serde(deserialize_with = "string_to_bool")]
    pub active: bool,
    pub note: Option<String>,
    pub address: Option<String>,
    pub value: Option<String>,
    #[serde(rename = "type")]
    pub typ: Option<CheckType>,
}

fn from_hex<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s: &str = Deserialize::deserialize(deserializer)?;
    from_hex_str::<D>(s)
}

fn from_hex_str<'de, D>(s: &str) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    let s = s.trim();
    let s = s
        .strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .unwrap_or(s);
    u32::from_str_radix(s, 16).map_err(|e| de::Error::custom(format!("invalid hex: {}", e)))
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
