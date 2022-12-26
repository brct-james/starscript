use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use spacedust::models::market_trade_good::Supply;
use spacedust::models::{Contract, MarketTradeGood, Survey};

/// Ledger holds economic state (e.g. credit balance, estimated net worth, contracts, faction influence, market values, survey results, etc.)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct Ledger {
    pub credit_balance: Balance,
    pub net_worth: Balance,
    pub contracts: HashMap<String, Contract>,
    pub market_data: HashMap<String, MarketData>,
    pub survey_data: HashMap<String, SurveyData>,
}

/// Balance holds a value as well as information related to when it was lash refreshed and how frequently it should be refreshed
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Balance {
    pub balance: i64,
    pub timestamp: u64,
    pub stale_after_seconds: u32,
}

impl Balance {
    pub fn new(starting_balance: i64, stale_after_seconds: Option<u32>) -> Self {
        Balance {
            balance: starting_balance,
            timestamp: 0,
            stale_after_seconds: stale_after_seconds.unwrap_or_default(),
        }
    }
}

impl Default for Balance {
    fn default() -> Balance {
        Balance {
            balance: 0,
            timestamp: 0,
            stale_after_seconds: 60,
        }
    }
}

/// MarketData holds market info for a given market including a historical record of price history for each item, estimated supply/demand, and last pulled price data
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct MarketData {
    pub waypoint_symbol: String,
    pub last_pulled_prices: HashMap<String, MarketTradeGood>,
    pub sell_price_history: HashMap<String, Vec<i32>>,
    pub buy_price_history: HashMap<String, Vec<i32>>,
    pub supply_history: HashMap<String, Vec<Supply>>,
    pub volume_history: HashMap<String, Vec<i32>>,
    pub timestamp: i64,
    pub stale_after: i64,
}

/// SurveyData holds survey info for each waypoint with active surveys as well as a historical record of what items were found at that waypoint and how rich the deposits were
/// Historical data is a HashMap where string is item symbol, value is Vec of integers where each entry corresponds to a single deposit such that if a deposit has two entries for Iron, the value is 2.
/// This should enable rarity estimation based on the length of the item vec over the survey count, and purity estimation based on the sum of the vec over its length
/// Deposit count should be incremented for each deposit, not each survey
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct SurveyData {
    pub waypoint_symbol: String,
    pub most_recent_surveys: Vec<Survey>,
    pub historical_survey_data: HashMap<String, Vec<u8>>,
    pub historical_survey_count: u64,
    pub timestamp: i64,
    pub stale_after: i64,
}

impl SurveyData {
    /// Replace most_recent_surveys and push to historical data
    pub fn insert(&mut self, new_surveys: Vec<Survey>) {
        // Push to historical_survey_data
        for survey in new_surveys.iter() {
            let mut resource_map: HashMap<String, u8> = Default::default();
            for deposit in survey.deposits.iter() {
                *resource_map.entry(deposit.symbol.to_string()).or_insert(0) += 1;
            }
            for (res_symbol, res_amt) in resource_map {
                self.historical_survey_data
                    .entry(res_symbol)
                    .or_insert(vec![])
                    .push(res_amt);
            }
            // Increment historical_deposit_count
            self.historical_survey_count += 1;
        }

        // Replace most_recent
        self.most_recent_surveys = new_surveys;
        // Set timestamp
        self.timestamp = chrono::Utc::now().timestamp();
    }

    /// Sort surveys by the approximate market value of their contents
    /// TODO: Include shipping distance in value calculation
    pub fn sort_surveys_by_market_data(&mut self, markets: &mut HashMap<String, (String, i64)>) {
        let good_values: HashMap<String, i64> = markets
            .iter()
            .map(|(k, m)| (k.clone(), m.1.clone()))
            .collect::<HashMap<String, i64>>();
        let survey_values: HashMap<String, i64> = self
            .most_recent_surveys
            .iter()
            .map(|s| {
                let mut sum = 0i64;
                for deposit in &s.deposits {
                    match good_values.get(&deposit.symbol) {
                        Some(gv) => sum += gv,
                        None => (),
                    }
                }
                return (s.signature.clone(), sum / s.deposits.len() as i64);
            })
            .collect::<HashMap<String, i64>>();
        self.most_recent_surveys.sort_by(|a, b| {
            let a_value = survey_values.get(&a.signature).unwrap();
            let b_value = survey_values.get(&b.signature).unwrap();
            return a_value.cmp(b_value).reverse();
        });
    }

    /// Sort surveys for the specified item symbols
    pub fn sort_surveys_for_items(&mut self, item_symbols: &Vec<String>) {
        let survey_values: HashMap<String, i64> = self
            .most_recent_surveys
            .iter()
            .map(|s| {
                let mut sum = 0i64;
                for deposit in &s.deposits {
                    if item_symbols.contains(&deposit.symbol) {
                        sum += 1;
                    }
                }
                return (s.signature.clone(), sum / s.deposits.len() as i64);
            })
            .collect::<HashMap<String, i64>>();
        self.most_recent_surveys.sort_by(|a, b| {
            let a_value = survey_values.get(&a.signature).unwrap();
            let b_value = survey_values.get(&b.signature).unwrap();
            return a_value.cmp(b_value).reverse();
        });
    }
}
