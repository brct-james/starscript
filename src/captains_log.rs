pub mod ledger;
pub mod shiprole;
pub mod shipstate;
pub mod shiptask;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use spacedust::models::{ship, Contract};
// use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CaptainsLog {
    pub ledger: ledger::Ledger,
    pub ships: HashMap<String, ControlledShip>,
}

impl CaptainsLog {
    pub fn new() -> Self {
        CaptainsLog {
            ledger: Default::default(),
            ships: Default::default(),
        }
    }
    pub fn upsert_ship(&mut self, ship: Box<ship::Ship>) {
        let mut role: shiprole::ShipRole = Default::default();
        let mut task_queue: Vec<shiptask::ShipTask> = Default::default();
        if self.ships.contains_key(&ship.symbol) {
            role = self.ships[&ship.symbol].role;
            task_queue = self.ships[&ship.symbol].task_queue.clone();
        }
        self.ships.insert(
            ship.symbol.to_string(),
            ControlledShip {
                ship: ship.clone(),
                role,
                state: shipstate::determine_ship_state(ship),
                task_queue,
                cooldown_timestamp: 0,
            },
        );
    }
    pub fn upsert_fleet(&mut self, fleet: Vec<ship::Ship>) {
        for ship in fleet {
            self.upsert_ship(Box::new(ship));
        }
    }
    pub fn upsert_ledger_contracts(&mut self, contracts: Vec<Contract>) {
        for contract in contracts {
            self.ledger
                .contracts
                .insert(contract.id.to_string(), contract);
        }
    }
    pub fn update_ship_role(&mut self, symbol: &String, new_role: shiprole::ShipRole) {
        self.ships.get_mut(symbol).unwrap().change_role(new_role);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ControlledShip {
    pub ship: Box<ship::Ship>,
    pub role: shiprole::ShipRole,
    pub state: shipstate::ShipState,
    pub task_queue: Vec<shiptask::ShipTask>, // Queue of tasks to do in order
    pub cooldown_timestamp: i64,
}

impl ControlledShip {
    pub fn change_role(&mut self, new_role: shiprole::ShipRole) {
        self.role = new_role;
    }
}
