use serde::{Deserialize, Serialize};
use spacedust::client::Client;
use spacedust::shared;
use std::collections::HashMap;
// use std::thread;
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;
// use tokio::sync::Mutex;

use chrono::prelude::*;
use tokio::time::{sleep, Duration};

// mod cadets;
// use crate::cadets::cadet;

// mod duties;
// use crate::duties::{Duty, DutyClass};

// mod captains_log;
// use crate::captains_log::{CaptainsLog, ShipWithCooldowns};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Stats {
    surveys: HashMap<String, i64>,
}

impl Stats {
    fn new() -> Stats {
        Stats {
            surveys: HashMap::new(),
        }
    }
}

fn save_stats(stats: Stats) -> Result<(), Box<dyn std::error::Error>> {
    let filename = "json/stats.json";
    let f = std::fs::OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open(filename)?;
    // write to file with serde
    serde_json::to_writer_pretty(f, &stats)?;

    Ok(())
}
fn load_stats() -> Result<Stats, std::io::Error> {
    let filename = "json/stats.json";
    let f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .open(filename)?;
    // serialize json as HashMap
    match serde_json::from_reader(f) {
        Ok(stats) => Ok(stats),
        Err(e) if e.is_eof() => Ok(Stats::new()),
        Err(e) => panic!("An error occurred: {}", e),
    }
}

fn save_agent(ship: shared::AgentInformation) -> Result<(), Box<dyn std::error::Error>> {
    let filename = "json/agent.json";
    let f = std::fs::OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open(filename)?;
    // write to file with serde
    serde_json::to_writer_pretty(f, &ship)?;

    Ok(())
}

fn save_ship(ship: shared::Ship) -> Result<(), Box<dyn std::error::Error>> {
    let filename = "json/ship.json";
    let f = std::fs::OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open(filename)?;
    // write to file with serde
    serde_json::to_writer_pretty(f, &ship)?;

    Ok(())
}

fn save_surveys(
    surveys: Vec<shared::Survey>,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let f = std::fs::OpenOptions::new()
        .truncate(true)
        .write(true)
        .create(true)
        .open(filename)?;
    // write to file with serde
    serde_json::to_writer_pretty(f, &surveys)?;

    Ok(())
}

fn load_surveys(filename: &str) -> Result<Vec<shared::Survey>, std::io::Error> {
    let f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .open(filename)?;
    // serialize json as HashMap
    match serde_json::from_reader(f) {
        Ok(surveys) => Ok(surveys),
        Err(e) if e.is_eof() => Ok(Vec::<shared::Survey>::new()),
        Err(e) => panic!("An error occurred: {}", e),
    }
}

fn sort_surveys_by_market_data(
    surveys: &mut Vec<shared::Survey>,
    markets: &mut HashMap<String, (String, i64)>,
) {
    let good_values: HashMap<String, i64> = markets
        .iter()
        .map(|(k, m)| (k.clone(), m.1.clone()))
        .collect::<HashMap<String, i64>>();
    let survey_values: HashMap<String, i64> = surveys
        .iter()
        .map(|s| {
            let mut sum = 0i64;
            for deposit in &s.deposits {
                match good_values.get(deposit) {
                    Some(gv) => sum += gv,
                    None => (),
                }
            }
            return (s.signature.clone(), sum / s.deposits.len() as i64);
        })
        .collect::<HashMap<String, i64>>();
    surveys.sort_by(|a, b| {
        let a_value = survey_values.get(&a.signature).unwrap();
        let b_value = survey_values.get(&b.signature).unwrap();
        return a_value.cmp(b_value).reverse();
    });
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Setup Game Client
    let http_client = spacedust::client::get_http_client(None);

    // // Register agent
    // let claim_agent_response = spacedust::client::claim_agent(
    //     http_client,
    //     "https://v2-0-0.alpha.spacetraders.io".to_string(),
    //     "<4-8 character string>".to_string(),
    //     "COMMERCE_REPUBLIC".to_string(),
    // )
    // .await.unwrap();

    // // Setup client using claimed agent
    // let client = Client::new(
    //     http_client,
    //     "https://v2-0-0.alpha.spacetraders.io".to_string(),
    //     claim_agent_response.data.agent.symbol,
    //     claim_agent_response.token,
    // );

    let _client = Client::new(
        http_client,
        "https://v2-0-0.alpha.spacetraders.io".to_string(),
        "GREEN".to_string(),
        "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZGVudGlmaWVyIjoiR1JFRU4iLCJpYXQiOjE2NDcwMTY1NjMsInN1YiI6ImFnZW50LXRva2VuIn0.S-9AfG_asd21tsdGf9TF-cwML32x-TFd-b2n9WT21CKA3gkS9qhR15Zng9I2chv92NRriUGUDVb3flc-nZfnbDrMK_iBUbHT7oLiUu1X4Rr9HumsHUdSEltpVGxTvRm6-0udRgLuy9ndoXCxomUsTruszqdRZ5BJb9-2OYcP_kU6FnYcERDGoKNn6jISmPCaSnSs8nCDw5dbSrDF16mAJiGozJlx9j1gDUHWzeQZF7k4fonPxcLPGQjSa4mKIMaYYCh5oATW3wMh5qnXb-iz-wiwHZ7aXd1jkmDVQzeFXYqLpNf1jjQOXXdqEcZ_lFe79Mgeg1vuNtJDZpPNh-KC7P7YdC_F-7DYA82x6uYDPN8bwxcPd5uNmw0lZr5_C0lUI_z8-igPurxDBLizwjBdMdjIaqY2YSjEV_zocRy-I-N_0c43Dc9a5zZoFFH0DPwFrR2c9pp3tSkFsRMHp86SVlASIDXCQlgLvlNoDORi79dVR9ap64JgK3z-ttoJ_v90".to_string(),
    );

    ////////////////////////////////////////////////////////////////////////////
    // Define Persistent Vars
    // TODO: Add recovering instead of return Ok(())
    let mut surveys: Vec<shared::Survey> = load_surveys("json/surveys.json").unwrap();
    let mut stats: Stats = load_stats().unwrap();

    // Collect loop
    loop {
        println!("--Starting Loop--");
        // Get Agent
        println!("Getting Agent");
        let agent: shared::AgentInformation;
        match _client.get_my_agent_details().await {
            Ok(res) => {
                println!("Got Agent");
                agent = res.data;
            }
            Err(res_err) => {
                println!("err {:?}", res_err);
                return Ok(());
            }
        }
        save_agent(agent.clone())?;

        // Get Ship
        println!("Getting Ship");
        let mut ship: shared::Ship;
        match _client.get_my_ship("GREEN-1".to_string()).await {
            Ok(res) => {
                println!("Got Ship, Located At: {:?}", res.data.location);
                ship = res.data;
            }
            Err(res_err) => {
                println!("err {:?}", res_err);
                return Ok(());
            }
        }
        save_ship(ship.clone())?;

        println!("Getting Travel Cooldown");
        let navigation_cooldown: shared::Navigation;
        match _client.ship_navigation_status("GREEN-1".to_string()).await {
            Ok(res) => {
                println!("Got Travel Cooldown");
                navigation_cooldown = res.data.navigation;
            }
            Err(res_err) => {
                println!("err {:?}", res_err);
                return Ok(());
            }
        }

        // Wait for travel to complete if traveling
        if navigation_cooldown.arrived_at.is_none() {
            let dur = (navigation_cooldown.duration_remaining.unwrap() * 1000) + 1000;
            println!("Waiting {}s for navigation cooldown", dur / 1000);
            sleep(Duration::from_millis(dur)).await;
            continue; // Go to next loop to refresh ship data
        }

        // Handle tasks at location
        let location = ship.location.as_ref().unwrap();
        if *location == "X1-OE-PM".to_string()
            || *location == "X1-OE-PM01".to_string()
            || *location == "X1-OE-PM02".to_string()
        {
            println!("At Port, Docking");
            // Dock
            match _client.dock_ship("GREEN-1".to_string()).await {
                Ok(_) => {
                    println!("Docked");
                }
                Err(res_err) => {
                    println!("err {:?}", res_err);
                    // return Ok(()); // Ignore error here, incase double dock due to restart or something
                }
            }

            // Sell Imported Cargo
            let market_imports: Vec<String>;
            match ship.location.as_ref().unwrap().as_str() {
                "X1-OE-PM" => market_imports = vec!["SILICON".to_string(), "QUARTZ".to_string()],
                "X1-OE-PM01" => {
                    market_imports = vec![
                        "IRON_ORE".to_string(),
                        "ALUMINUM_ORE".to_string(),
                        "COPPER_ORE".to_string(),
                    ]
                }
                "X1-OE-PM02" => {
                    market_imports = vec!["ALUMINUM".to_string(), "ELECTRONICS".to_string()]
                }
                _ => {
                    println!(
                        "Error, unexpected location: {:#?}",
                        ship.location.as_ref().unwrap()
                    );
                    return Ok(());
                }
            }
            let mut txs: Vec<String> = vec![];
            for good in &ship.cargo {
                for import_good in &market_imports {
                    if good.trade_symbol.to_string() == *import_good {
                        // Sell
                        println!("Selling Good {}", import_good);
                        match _client
                            .sell_cargo(
                                "GREEN-1".to_string(),
                                good.trade_symbol.to_string(),
                                good.units,
                            )
                            .await
                        {
                            Ok(res) => {
                                println!(
                                    "Sold {} units of {} for {} total credits",
                                    res.data.units, res.data.trade_symbol, res.data.credits
                                );
                            }
                            Err(res_err) => {
                                println!("err {:?}", res_err);
                                return Ok(());
                            }
                        }
                        // Add to TX list
                        txs.push(good.trade_symbol.to_string());
                    }
                }
            }

            // Remove TX'd goods from cargo
            for tx in txs {
                let index = ship
                    .cargo
                    .iter()
                    .position(|g| g.trade_symbol == tx)
                    .unwrap();
                ship.cargo.remove(index);
            }

            // Refuel if necessary
            if ship.fuel <= 100 {
                // Buy Fuel
                match _client.refuel_ship("GREEN-1".to_string()).await {
                    Ok(_) => {
                        println!("Refueled");
                    }
                    Err(res_err) => {
                        println!("err {:?}", res_err);
                        return Ok(());
                    }
                }
            }

            println!("Business Concluded, Moving to Orbit");
            // Orbit
            match _client.orbit_ship("GREEN-1".to_string()).await {
                Ok(_) => {
                    println!("Orbited");
                }
                Err(res_err) => {
                    println!("err {:?}", res_err);
                    // return Ok(()); // Ignore error here, incase double dock due to restart or something
                }
            }

            // If Cargo Still Contains Sellable Goods, Continue to Next Location
            if ship.cargo.len() > 0 {
                println!(
                    "Cargo Still Contains Sellable Goods {:#?}, Continuing to Next Location",
                    ship.cargo
                        .iter()
                        .map(|g| format!("{}: {}, ", g.trade_symbol, g.units))
                        .collect::<String>()
                );
                let destination_symbol;
                if ship
                    .cargo
                    .iter()
                    .any(|c| c.trade_symbol == "SILICON" || c.trade_symbol == "QUARTZ")
                {
                    // Send to X1-OE-PM
                    destination_symbol = "X1-OE-PM".to_string();
                } else if ship.cargo.iter().any(|c| {
                    c.trade_symbol == "IRON_ORE"
                        || c.trade_symbol == "ALUMINUM_ORE"
                        || c.trade_symbol == "COPPER_ORE"
                }) {
                    // Send to X1-OE-PM01
                    destination_symbol = "X1-OE-PM01".to_string();
                } else if ship
                    .cargo
                    .iter()
                    .any(|c| c.trade_symbol == "ALUMINUM" || c.trade_symbol == "ELECTRONICS")
                {
                    // Send to X1-OE-PM02
                    destination_symbol = "X1-OE-PM02".to_string();
                } else {
                    // Vent any other cargo as it can't be sold in system then continue to next loop
                    for good in ship.cargo {
                        match _client
                            .jettison_cargo(
                                "GREEN-1".to_string(),
                                good.trade_symbol.to_string(),
                                good.units,
                            )
                            .await
                        {
                            Ok(_) => {
                                println!("Jettisoned Cargo: {}: {}", good.trade_symbol, good.units);
                            }
                            Err(res_err) => {
                                println!("err {:?}", res_err);
                                return Ok(());
                            }
                        }
                    }
                    continue;
                }
                println!("Destination: {}", destination_symbol);
                let navigate_response: shared::Navigation;
                match _client
                    .navigate_ship("GREEN-1".to_string(), destination_symbol.to_string())
                    .await
                {
                    Ok(res) => {
                        println!("Began Travel to {}", destination_symbol.to_string());
                        navigate_response = res.data.navigation;
                    }
                    Err(res_err) => {
                        println!("err {:?}", res_err);
                        return Ok(());
                    }
                }
                let dur = (navigate_response.duration_remaining.unwrap() * 1000) + 1000;
                println!("Waiting {}s for navigation cooldown", dur / 1000);
                sleep(Duration::from_millis(dur)).await;
                continue; // Go to next loop to refresh ship data
            }

            // If Cargo Emptied, Go Back to Harvest More
            if ship.cargo.len() == 0 {
                println!("Cargo Empty, Go Harvest");
                let destination_symbol = "X1-OE-A005".to_string();
                let navigate_response: shared::Navigation;
                match _client
                    .navigate_ship("GREEN-1".to_string(), destination_symbol.to_string())
                    .await
                {
                    Ok(res) => {
                        println!("Began Travel to {}", destination_symbol.to_string());
                        navigate_response = res.data.navigation;
                    }
                    Err(res_err) => {
                        println!("err {:?}", res_err);
                        return Ok(());
                    }
                }
                let dur = (navigate_response.duration_remaining.unwrap() * 1000) + 1000;
                println!("Waiting {}s for navigation cooldown", dur / 1000);
                sleep(Duration::from_millis(dur)).await;
                continue; // Go to next loop to refresh ship data
            }
        } else if *location == "X1-OE-A005".to_string() {
            println!("At Asteroid Field");
            // If Cargo Full or Nearly Full, Go Sell
            let cargo_used = ship.cargo.iter().map(|g| g.units).sum::<i64>();
            println!("Cargo Used: {} of {}", cargo_used, ship.stats.cargo_limit);
            if cargo_used + 10 >= ship.stats.cargo_limit.into() {
                println!("Cargo Full or Nearly Full, Go Sell");
                let destination_symbol;
                if ship
                    .cargo
                    .iter()
                    .any(|c| c.trade_symbol == "SILICON" || c.trade_symbol == "QUARTZ")
                {
                    // Send to X1-OE-PM
                    destination_symbol = "X1-OE-PM".to_string();
                } else if ship.cargo.iter().any(|c| {
                    c.trade_symbol == "IRON_ORE"
                        || c.trade_symbol == "ALUMINUM_ORE"
                        || c.trade_symbol == "COPPER_ORE"
                }) {
                    // Send to X1-OE-PM01
                    destination_symbol = "X1-OE-PM01".to_string();
                } else if ship
                    .cargo
                    .iter()
                    .any(|c| c.trade_symbol == "ALUMINUM" || c.trade_symbol == "ELECTRONICS")
                {
                    // Send to X1-OE-PM02
                    destination_symbol = "X1-OE-PM02".to_string();
                } else {
                    // Vent any other cargo as it can't be sold in system then continue to next loop
                    for good in ship.cargo {
                        match _client
                            .jettison_cargo(
                                "GREEN-1".to_string(),
                                good.trade_symbol.to_string(),
                                good.units,
                            )
                            .await
                        {
                            Ok(_) => {
                                println!("Jettisoned Cargo: {}: {}", good.trade_symbol, good.units);
                            }
                            Err(res_err) => {
                                println!("err {:?}", res_err);
                                return Ok(());
                            }
                        }
                    }
                    continue;
                }
                let navigate_response: shared::Navigation;
                match _client
                    .navigate_ship("GREEN-1".to_string(), destination_symbol.to_string())
                    .await
                {
                    Ok(res) => {
                        println!("Began Travel to {}", destination_symbol.to_string());
                        navigate_response = res.data.navigation;
                    }
                    Err(res_err) => {
                        println!("err {:?}", res_err);
                        return Ok(());
                    }
                }
                let dur = (navigate_response.duration_remaining.unwrap() * 1000) + 1000;
                println!("Waiting {}s for navigation cooldown", dur / 1000);
                sleep(Duration::from_millis(dur)).await;
                continue; // Go to next loop to refresh ship data
            }

            // Check Extract Cooldown first to prevent stale active_survey after waiting on extract_cooldown
            println!("Getting Extract Cooldown");
            let extract_cooldown: shared::Cooldown;
            match _client.get_extract_cooldown("GREEN-1".to_string()).await {
                Ok(res) => {
                    println!("Got Extract Cooldown");
                    extract_cooldown = res.data.cooldown;
                }
                Err(res_err) => {
                    println!("err {:?}", res_err);
                    return Ok(());
                }
            }

            // If On Cooldown, Wait
            if extract_cooldown.duration > 0 {
                // Wait till can get survey
                let dur = (extract_cooldown.duration * 1000) + 1000;
                println!("Waiting {}s for extract cooldown", dur / 1000);
                sleep(Duration::from_millis(dur)).await;
            }

            println!("Remove Expired Surveys");
            // Remove expired surveys from list
            let now = Utc::now().timestamp() + 5; // Buffer 5s to try and prevent errors
            surveys.retain(|s| {
                DateTime::parse_from_rfc3339(&s.expiration.as_str())
                    .unwrap()
                    .timestamp()
                    >= now
            });
            // If no surveys remain, attempt to get more
            if surveys.len() == 0 {
                println!("No Surveys Remain, Need to Re-Survey");
                // Check survey cooldown
                println!("Getting Survey Cooldown");
                let survey_cooldown: shared::Cooldown;
                match _client.get_survey_cooldown("GREEN-1".to_string()).await {
                    Ok(res) => {
                        println!("Got Survey Cooldown");
                        survey_cooldown = res.data.cooldown;
                    }
                    Err(res_err) => {
                        println!("err {:?}", res_err);
                        return Ok(());
                    }
                }
                if survey_cooldown.duration > 0 {
                    let temp_surveys = load_surveys("json/temp_surveys.json").unwrap();
                    if temp_surveys.len() == 0 {
                        // temp surveys not found, continue
                        println!("Nothing in temp_surveys.json");
                        if survey_cooldown.duration >= 120 && cargo_used >= 200 {
                            // Rather than waiting fruitlessly for more surveys, go sell cargo now instead since have over 70%
                            println!("Survey Cooldown {} >= 120, and Cargo Used {} >= 200, Go Sell Now Rather than Waiting Around", survey_cooldown.duration, cargo_used);
                            let destination_symbol;
                            if ship
                                .cargo
                                .iter()
                                .any(|c| c.trade_symbol == "SILICON" || c.trade_symbol == "QUARTZ")
                            {
                                // Send to X1-OE-PM
                                destination_symbol = "X1-OE-PM".to_string();
                            } else if ship.cargo.iter().any(|c| {
                                c.trade_symbol == "IRON_ORE"
                                    || c.trade_symbol == "ALUMINUM_ORE"
                                    || c.trade_symbol == "COPPER_ORE"
                            }) {
                                // Send to X1-OE-PM01
                                destination_symbol = "X1-OE-PM01".to_string();
                            } else if ship.cargo.iter().any(|c| {
                                c.trade_symbol == "ALUMINUM" || c.trade_symbol == "ELECTRONICS"
                            }) {
                                // Send to X1-OE-PM02
                                destination_symbol = "X1-OE-PM02".to_string();
                            } else {
                                // Vent any other cargo as it can't be sold in system then continue to next loop
                                for good in ship.cargo {
                                    match _client
                                        .jettison_cargo(
                                            "GREEN-1".to_string(),
                                            good.trade_symbol.to_string(),
                                            good.units,
                                        )
                                        .await
                                    {
                                        Ok(_) => {
                                            println!(
                                                "Jettisoned Cargo: {}: {}",
                                                good.trade_symbol, good.units
                                            );
                                        }
                                        Err(res_err) => {
                                            println!("err {:?}", res_err);
                                            return Ok(());
                                        }
                                    }
                                }
                                continue;
                            }
                            let navigate_response: shared::Navigation;
                            match _client
                                .navigate_ship(
                                    "GREEN-1".to_string(),
                                    destination_symbol.to_string(),
                                )
                                .await
                            {
                                Ok(res) => {
                                    println!("Began Travel to {}", destination_symbol.to_string());
                                    navigate_response = res.data.navigation;
                                }
                                Err(res_err) => {
                                    println!("err {:?}", res_err);
                                    return Ok(());
                                }
                            }
                            let dur = (navigate_response.duration_remaining.unwrap() * 1000) + 1000;
                            println!("Waiting {}s for navigation cooldown", dur / 1000);
                            sleep(Duration::from_millis(dur)).await;
                            continue; // Go to next loop to refresh ship data
                        }
                        // Wait till can get survey
                        let dur = (survey_cooldown.duration * 1000) + 1000;
                        println!("Waiting {}s for survey cooldown", dur / 1000);
                        sleep(Duration::from_millis(dur)).await;
                        // Get new set of surveys
                        println!("Conducting Survey");
                        match _client.survey_surroundings("GREEN-1".to_string()).await {
                            Ok(res) => {
                                println!("Got Surveys");
                                surveys = res.data.surveys;
                            }
                            Err(res_err) => {
                                println!("err {:?}", res_err);
                                return Ok(());
                            }
                        }
                        // save temp_surveys incase error or crash while sorting
                        println!("Save temp_surveys.json incase error or crash while sorting");
                        save_surveys(surveys.clone(), "json/temp_surveys.json").unwrap();
                    } else {
                        println!("Recovering from Incomplete Sort Step, Loaded temp_surveys.json");
                        surveys = temp_surveys;
                    }
                } else {
                    // Get new set of surveys
                    println!("Conducting Survey");
                    match _client.survey_surroundings("GREEN-1".to_string()).await {
                        Ok(res) => {
                            println!("Got Surveys");
                            surveys = res.data.surveys;
                        }
                        Err(res_err) => {
                            println!("err {:?}", res_err);
                            return Ok(());
                        }
                    }
                }

                // Order surveys vec by value
                // TODO: get market symbols automatically
                // TODO: select where to sell goods based on best prices (in case some systems have better prices at a certain waypoint for the same good)
                println!("Getting Market Conditions");
                let mut markets = HashMap::<String, (String, i64)>::new();
                let m_symbols: [&str; 3] = ["X1-OE-PM", "X1-OE-PM01", "X1-OE-PM02"];
                for symb in m_symbols {
                    let market_string = symb.to_string();
                    let system_string =
                        market_string.split("-").collect::<Vec<&str>>()[0..2].join("-");
                    println!(
                        "System String: {}, Market String: {}",
                        system_string, market_string
                    );
                    match _client
                        .get_system_market(system_string, market_string.clone())
                        .await
                    {
                        Ok(res) => {
                            println!("Got Market {}", &market_string);
                            for import in res.data.imports {
                                if markets.contains_key(&import.trade_symbol) {
                                    // Check if stored price is better than price here
                                    if markets.get(&import.trade_symbol).unwrap().1 > import.price {
                                        continue;
                                    }
                                }
                                *markets
                                    .entry(import.trade_symbol.to_string())
                                    .or_insert((market_string.clone(), import.price)) =
                                    (market_string.clone(), import.price);
                            }
                        }
                        Err(res_err) => {
                            println!("err {:?}", res_err);
                            return Ok(());
                        }
                    }
                }
                println!(
                    "Got Markets: {:#?}",
                    markets
                        .iter()
                        .map(|(g, m)| format!("{}: {} @ {}", g, m.1, m.0))
                        .collect::<Vec<String>>()
                );
                println!(
                    "Ordering Surveys, starting order: {:#?}",
                    surveys
                        .iter()
                        .map(|s| s.signature.to_string())
                        .collect::<Vec<String>>()
                );
                sort_surveys_by_market_data(&mut surveys, &mut markets);
                println!(
                    "Ordered Surveys, new order: {:#?}",
                    surveys
                        .iter()
                        .map(|s| s.signature.to_string())
                        .collect::<Vec<String>>()
                );
                // surveys.sort_by_key(|s| (s.deposits.iter().filter(|&d| *d == best_goods[0]).count(), s.deposits.iter().filter(|&d| *d == best_goods[1]).count(), s.deposits.iter().filter(|&d| *d == best_goods[2]).count(), s.deposits.iter().filter(|&d| *d == best_goods[3]).count()));
            }
            // Save Surveys
            save_surveys(surveys.clone(), "json/surveys.json").unwrap();
            save_surveys(vec![], "json/temp_surveys.json").unwrap();

            // Active Survey is selected from remaining sorted vec
            let active_survey = surveys[0].clone();
            let key = format!(
                "{} | {:?} | {}",
                active_survey.signature, active_survey.deposits, active_survey.expiration
            );
            println!("Selected Active Survey {}", key);
            let as_timestamp = DateTime::parse_from_rfc3339(&active_survey.expiration.as_str())
                .unwrap()
                .timestamp();
            println!(
                "Active Survey has timestamp: {} >= {} ? {}",
                as_timestamp,
                now,
                as_timestamp >= now
            );

            // TODO: Loop this directly here updating ship from extraction response
            // Use Active Survey to Extract Materials
            println!(
                "Extracting from Survey with Deposits: {:?}",
                active_survey.deposits
            );
            let extraction_results: shared::ExtractData;
            match _client
                .extract_resources("GREEN-1".to_string(), Some(active_survey.clone()))
                .await
            {
                Ok(res) => {
                    println!("Got Extraction Results");
                    extraction_results = res.data;
                }
                Err(err) => {
                    println!("err {:?}", err);
                    surveys.remove(0);
                    continue;
                    // return Ok(());
                    // match err {
                    //     spacedust::errors::SpaceTradersClientError::JsonParse(serde_json::Error { err }) => {
                    //         let str_err: dyn std::error::Error = *err.into::<std::error::Error>();
                    //         println!("Exhausted Error {}", *err.to_string());
                    //         return Ok(());
                    //     }
                    //     _ => {
                    //         println!("err {:?}", err);
                    //         return Ok(());
                    //     }
                    // }
                }
            }
            println!(
                "Extraction Yield: {:#?}",
                extraction_results.extraction.extract_yield
            );
            *stats.surveys.entry(key.to_string()).or_insert(0) += 1;
            if stats.surveys.get(&key).unwrap() >= &20i64 {
                surveys.remove(0);
            }
            save_stats(stats.clone())?;
        } else {
            println!("Error, At Unexpected Location: {}", location);
            return Ok(());
        }
    }
}
