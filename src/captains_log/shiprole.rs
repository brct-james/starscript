use serde::{Deserialize, Serialize};

/// The current role of the ship
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum ShipRole {
    #[serde(rename = "UNASSIGNED")]
    Unassigned,
    #[serde(rename = "MINER")]
    Miner,
    #[serde(rename = "TRADER")]
    Trader,
    #[serde(rename = "FIGHTER")]
    Fighter,
    #[serde(rename = "EXPLORER")]
    Explorer,
    #[serde(rename = "MARKET_SCOUT")]
    MarketScout,
}

impl ToString for ShipRole {
    fn to_string(&self) -> String {
        match self {
            Self::Unassigned => String::from("UNASSIGNED"),
            Self::Miner => String::from("MINER"),
            Self::Trader => String::from("TRADER"),
            Self::Fighter => String::from("FIGHTER"),
            Self::Explorer => String::from("EXPLORER"),
            Self::MarketScout => String::from("MARKET_SCOUT"),
        }
    }
}

impl Default for ShipRole {
    fn default() -> ShipRole {
        Self::Unassigned
    }
}
