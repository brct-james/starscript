use spacedust::shared::{Navigation, Ship};
// use chrono::prelude::*;

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct CaptainsLog {
    pub ships: Vec<ShipWithCooldowns>,
}

impl CaptainsLog {
    pub fn new() -> Self {
        CaptainsLog { ships: vec![] }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct ShipWithCooldowns {
    pub ship: Ship,
    pub navigation: Option<Navigation>,
}
