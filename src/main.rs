// Uncategorized Imports
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;
// use tokio::sync::Mutex;

// use tokio::time::{sleep, Duration};

// API
use spacedust::apis::agents_api::get_my_agent;
use spacedust::apis::configuration::Configuration;
use spacedust::apis::default_api::register;
use spacedust::models::register_request::{Faction, RegisterRequest};

// Modules
// mod cadets;

// mod duties;

// mod captains_log;

// mod astronomicon;

// mod signaller;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Configuration
    let mut conf = Configuration::new();

    // Create Register Request
    let reg_req = RegisterRequest::new(Faction::Cosmic, "test1454121".to_string());

    // Register Agent
    let register_response = register(&conf, Some(reg_req)).await;

    match register_response {
        Ok(res) => {
            println!("{:#?}", res);
            // Update Config with Agent Token
            conf.bearer_access_token = Some(res.data.token);
        }
        Err(err_res) => {
            panic!("{:#?}", err_res);
        }
    }

    // Get Agent Details to Confirm Working
    match get_my_agent(&conf).await {
        Ok(res) => {
            println!("{:#?}", res);
            // Print Symbol
            println!("My Symbol: {:#?}", res.data.symbol);
        }
        Err(err_res) => {
            panic!("{:#?}", err_res);
        }
    }

    Ok(())
}
