use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Defines a structure for retaining information on the galaxy, its systems, and waypoints
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Astronomicon {
    pub systems: String, // TODO: Use a tree or some other graph structure with headquarters as root
    pub waypoints: HashMap<String, String>, // All waypoints ever visited
}

impl Astronomicon {
    pub fn new() -> Self {
        Astronomicon::default()
    }
}
