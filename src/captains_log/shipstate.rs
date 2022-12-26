use serde::{Deserialize, Serialize};
use spacedust::models::{ship, ship_nav_status};

pub fn determine_ship_state(ship: Box<ship::Ship>) -> ShipState {
    let mut _state = ShipState::default();
    match ship.nav.status {
        ship_nav_status::ShipNavStatus::InTransit => _state = ShipState::InTransit,
        ship_nav_status::ShipNavStatus::InOrbit => _state = ShipState::InOrbit,
        ship_nav_status::ShipNavStatus::Docked => _state = ShipState::Docked,
    }
    return _state;
}

/// The current status of the ship
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum ShipState {
    #[serde(rename = "UNKNOWN")]
    Unknown,
    #[serde(rename = "IN_TRANSIT")]
    InTransit,
    #[serde(rename = "IN_ORBIT")]
    InOrbit,
    #[serde(rename = "DOCKED")]
    Docked,
    #[serde(rename = "EXTRACTING")]
    Extracting,
}

impl ToString for ShipState {
    fn to_string(&self) -> String {
        match self {
            Self::Unknown => String::from("UNKNOWN"),
            Self::InTransit => String::from("IN_TRANSIT"),
            Self::InOrbit => String::from("IN_ORBIT"),
            Self::Docked => String::from("DOCKED"),
            Self::Extracting => String::from("EXTRACTING"),
        }
    }
}

impl Default for ShipState {
    fn default() -> ShipState {
        Self::Unknown
    }
}
