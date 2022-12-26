use serde::{Deserialize, Serialize};

/// The current status of the ship
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum TaskID {
    #[serde(rename = "IDLE")]
    Idle,
    #[serde(rename = "NAVIGATE")]
    Navigate,
    #[serde(rename = "BUY")]
    Buy,
    #[serde(rename = "SELL")]
    Sell,
    #[serde(rename = "SURVEY")]
    Survey,
    #[serde(rename = "SCAN")]
    Scan,
    #[serde(rename = "EXTRACT")]
    Extract,
    #[serde(rename = "DEPLOY")]
    Deploy,
    #[serde(rename = "BUY_SHIP")]
    BuyShip,
}

impl ToString for TaskID {
    fn to_string(&self) -> String {
        match self {
            Self::Idle => String::from("IDLE"),
            Self::Navigate => String::from("IN_TRANSIT"),
            Self::Buy => String::from("IN_ORBIT"),
            Self::Sell => String::from("DOCKED"),
            Self::Survey => String::from("DOCKED"),
            Self::Scan => String::from("DOCKED"),
            Self::Extract => String::from("DOCKED"),
            Self::Deploy => String::from("DOCKED"),
            Self::BuyShip => String::from("DOCKED"),
        }
    }
}

impl Default for TaskID {
    fn default() -> TaskID {
        Self::Idle
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShipTask {
    task_id: TaskID,
    // Should also contain information like route, market order, etc. necessary for task completion
}
