use std::collections::HashMap;

pub struct RulesManager {
    staleness_rules: StalenessRules,
}

impl RulesManager {
    pub fn new() -> Self {
        let rules = load_rules_json("json/settings/staleness_rules.json");

        Self {
            staleness_rules: rules,
        }
    }

    pub fn get_staleness_rules(&self) -> &StalenessRules {
        &self.staleness_rules
    }

    pub fn _get_staleness_rule(&self, rule_name: &String) -> Option<i64> {
        let result = self.staleness_rules.get(rule_name);
        match result {
            Some(rule) => {
                return Some(rule.clone());
            }
            None => {
                return None;
            }
        }
    }
}

fn load_rules_json(filename: &str) -> StalenessRules {
    let f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .open(filename)
        .unwrap();
    // serialize json as HashMap
    match serde_json::from_reader(f) {
        Ok(rules) => rules,
        Err(e) => {
            if !e.is_eof() {
                panic!("Error while loading/deserializing staleness rules json. Filename: {}, Error: {:#?}",
                    filename,
                    e,
                );
            }
            panic!(
                "Staleness rules file empty! Filename: {}, Error: {:#?}",
                filename, e,
            );
        }
    }
}

/// Defines a type for serializing/deserializing staleness rules
pub type StalenessRules = HashMap<String, i64>;
