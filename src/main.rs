// Uncategorized Imports
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;
// use tokio::sync::Mutex;

use mongodb::Collection;
// use tokio::time::{sleep, Duration};
use mongodb::bson::{doc, from_bson, to_document, Bson, Document};
use mongodb::options::ReplaceOptions;
use mongodb::{options::ClientOptions, Client};

use dotenv;
use std::env;

use chrono::prelude::Utc;

// API
use spacedust::apis::agents_api::get_my_agent;
use spacedust::apis::configuration::Configuration;
// use spacedust::apis::default_api::register;
// use spacedust::models::register_request::{Faction, RegisterRequest};

// Modules
// mod cadets;

// mod duties;

// mod captains_log;

// mod astronomicon;

// mod signaller;

mod cadet;
use cadet::{admiral::Admiral, astropath::Astropath, factor::Factor, navigator::Navigator};

mod log;
use crate::log::{Log, LogSeverity, ProcessState, ProcessStatus};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get Mongo Creds from Env
    dotenv::from_filename("mongo_secrets.env").ok();

    // Parse a mongo connection string into an options struct
    let connection_string = format!(
        "mongodb://{}:{}@localhost:27018",
        env::var("MONGO_INITDB_ROOT_USERNAME")?,
        env::var("MONGO_INITDB_ROOT_PASSWORD")?
    );
    let mut client_options = ClientOptions::parse(connection_string).await?;

    // Set client app name
    client_options.app_name = Some("starscript-client".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options)?;

    // Get a handle to each database
    let db = client.database("starscript");
    let log_db = client.database("starscript-logs");

    // Create SPMC command channel, used to gracefully shutdown threads
    let (cmd_tx, cmd_rx) = tokio::sync::watch::channel("run".to_string());
    cmd_tx.send("run".to_string()).unwrap();

    // Get process status collection
    let process_status_collection = log_db.collection("process_status");
    process_status_collection.drop(None).await.unwrap();

    // Create MPMC log channel, used to route logs to the correct service
    let (log_tx, log_rx) = tokio::sync::broadcast::channel(8);

    // Get log collections for each level
    let routine_log_collection = log_db.collection("routine");
    routine_log_collection.drop(None).await.unwrap();
    let priority_log_collection = log_db.collection("priority");
    priority_log_collection.drop(None).await.unwrap();
    let critical_log_collection = log_db.collection("critical");
    critical_log_collection.drop(None).await.unwrap();

    // Create log objects
    let mut routine_log = Log::new(
        routine_log_collection.clone(),
        "ROUTINE".to_string(),
        vec![
            LogSeverity::Routine,
            LogSeverity::Priority,
            LogSeverity::Critical,
        ],
        cmd_rx.clone(),
        log_rx,
    );
    let mut priority_log = Log::new(
        priority_log_collection.clone(),
        "PRIORITY".to_string(),
        vec![LogSeverity::Priority, LogSeverity::Critical],
        cmd_rx.clone(),
        log_tx.subscribe(),
    );
    let mut critical_log = Log::new(
        critical_log_collection.clone(),
        "CRITICAL".to_string(),
        vec![LogSeverity::Critical],
        cmd_rx.clone(),
        log_tx.subscribe(),
    );

    // Set Process Status for Logs to STARTING
    process_status_collection
        .insert_one(
            to_document(&ProcessStatus::new(
                "LOG_ROUTINE".to_string(),
                ProcessState::STARTING,
            ))
            .unwrap(),
            None,
        )
        .await?;
    process_status_collection
        .insert_one(
            to_document(&ProcessStatus::new(
                "LOG_PRIORITY".to_string(),
                ProcessState::STARTING,
            ))
            .unwrap(),
            None,
        )
        .await?;
    process_status_collection
        .insert_one(
            to_document(&ProcessStatus::new(
                "LOG_CRITICAL".to_string(),
                ProcessState::STARTING,
            ))
            .unwrap(),
            None,
        )
        .await?;

    // Set Pre-Spawn Timestamp
    let pre_log_spawn_timestamp = Utc::now();

    // Spawn threads for each log
    let routine_log_process_status_collection = process_status_collection.clone();
    let priority_log_process_status_collection = process_status_collection.clone();
    let critical_log_process_status_collection = process_status_collection.clone();
    tokio::spawn(async move {
        routine_log
            .initialize(routine_log_process_status_collection)
            .await
    });
    tokio::spawn(async move {
        priority_log
            .initialize(priority_log_process_status_collection)
            .await
    });
    tokio::spawn(async move {
        critical_log
            .initialize(critical_log_process_status_collection)
            .await
    });

    // Wait for Initialization Status to be Ready for Each in DB
    let mut log_processes_ready = false;
    let mut routine_process_state = "STARTING".to_string();
    let mut priority_process_state = "STARTING".to_string();
    let mut critical_process_state = "STARTING".to_string();

    while log_processes_ready == false {
        print!(
            "\rWaiting for Logs to be State READY | Routine: {}, Priority: {}, Critical: {} | Time Elapsed: {}",
            routine_process_state,
            priority_process_state,
            critical_process_state,
            Utc::now()
                .signed_duration_since(pre_log_spawn_timestamp)
                .to_string()
        );

        // Check Statuses
        let routine_process_found = process_status_collection
            .find_one(Some(doc! {"process_id": "LOG_ROUTINE"}), None)
            .await
            .unwrap();
        match routine_process_found {
            Some(ref document) => {
                routine_process_state = document.get_str("state").unwrap().to_string()
            }
            None => (),
        }
        let priority_process_found = process_status_collection
            .find_one(Some(doc! {"process_id": "LOG_PRIORITY"}), None)
            .await
            .unwrap();
        match priority_process_found {
            Some(ref document) => {
                priority_process_state = document.get_str("state").unwrap().to_string()
            }
            None => (),
        }
        let critical_process_found = process_status_collection
            .find_one(Some(doc! {"process_id": "LOG_CRITICAL"}), None)
            .await
            .unwrap();
        match critical_process_found {
            Some(ref document) => {
                critical_process_state = document.get_str("state").unwrap().to_string()
            }
            None => (),
        }

        log_processes_ready = routine_process_state == "READY".to_string()
            && priority_process_state == "READY".to_string()
            && critical_process_state == "READY".to_string();
    }
    print!(
        "\rWaiting for Logs to be State READY | Routine: {}, Priority: {}, Critical: {} | Time Elapsed: {}\n",
        routine_process_state,
        priority_process_state,
        critical_process_state,
        Utc::now()
            .signed_duration_since(pre_log_spawn_timestamp)
            .to_string()
    );
    println!("All Logs Finished Initializing at Startup");

    // Attempt Registration or Login
    // TODO: Implement Here

    // Get Agent Symbol
    let agent_symbol: String = "".to_string();

    // Create Cadets
    // TODO: Implement Process_Status use here
    // TODO: Figure out why after hitting 'All Non-Ensign Cadets Initialized' the program seems to hang, also nothing is being logged to routine in the intialize step of these?
    let navigator = Navigator::new(
        "NAV".to_string(),
        agent_symbol.to_string(),
        cmd_rx.clone(),
        log_tx.clone(),
    );
    let factor = Factor::new(
        "FAC".to_string(),
        agent_symbol.to_string(),
        cmd_rx.clone(),
        log_tx.clone(),
    );
    let astropath = Astropath::new(
        "AST".to_string(),
        agent_symbol.to_string(),
        cmd_rx.clone(),
        log_tx.clone(),
    );
    let admiral = Admiral::new(
        "ADM".to_string(),
        agent_symbol.to_string(),
        cmd_rx.clone(),
        log_tx.clone(),
    );

    // Spawn threads for each cadet
    tokio::spawn(async move { navigator.initialize().await });
    tokio::spawn(async move { factor.initialize().await });
    tokio::spawn(async move { astropath.initialize().await });
    tokio::spawn(async move { admiral.initialize().await });
    println!("All Non-Ensign Cadets Initialized");

    // Get handles for all the relevant collections
    // let account: Collection<Document> = db.collection("account");
    let agent: Collection<Document> = db.collection("agent");
    // let contracts: Collection<Document> = db.collection("contracts");
    // let fleet: Collection<Document> = db.collection("fleet");
    // let systems: Collection<Document> = db.collection("fleet");
    // let waypoints: Collection<Document> = db.collection("fleet");

    // // // Construct simple document
    // // let data = TestData {
    // //     question: "What is 42?".to_string(),
    // //     answer: 42,
    // // };
    // // let document = to_document(&data)?;
    // // collection.insert_one(document, None).await?;

    // // List the names of the collections in that database.
    // for collection_name in db.list_collection_names(None).await? {
    //     println!("{}", collection_name);
    // }

    // // Get Data
    // let result = collection.find_one(doc! {"answer": 42}, None).await?;

    // match result {
    //     Some(ref document) => {
    //         println!(
    //             "Found question {} for answer 42",
    //             document.get_str("question")?
    //         );
    //     }
    //     None => {
    //         println!("Could not find question for answer 42");
    //     }
    // }

    // Create Configuration
    let mut conf = Configuration::new();

    // // Create Register Request
    // let reg_req = RegisterRequest::new(Faction::Cosmic, "test144121".to_string());

    // // Register Agent
    // let register_response = register(&conf, Some(reg_req)).await;

    // match register_response {
    //     Ok(res) => {
    //         println!("{:#?}", res);
    //         // Update Config with Agent Token
    //         conf.bearer_access_token = Some(res.data.token);
    //     }
    //     Err(err_res) => {
    //         panic!("{:#?}", err_res);
    //     }
    // }

    conf.bearer_access_token = Some("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZGVudGlmaWVyIjoiVEVTVDE0NDEyMSIsImlhdCI6MTY3NzM1MzkzNywic3ViIjoiYWdlbnQtdG9rZW4ifQ.sKH_plunZQl8qbu_wR1a65beysxCpYrHwmZwXadlT9rzkHhxXcDf7WczbOFHjD9mjas88lL2k_BJWJ5GCfn3HAoQCTIjHA3IbSh7ugHptjVBByRFSRzV2yddTZgFKKgc7nD7128w23w7t2mYAZ9q-1m8fSwMqrKPP7_RgDMtrt6nOW3VHyS5aPSssiu8bsdDqMzUAkEwy1N2SYVZTIOQL6IEq5OYljY9QONy2isXkJSGvWlFGNlZH6WQAYQeXKc5fvIUv5rG4hH0oGjeb6ncnyZUQjEmIlDGoRWsiXWgUS0iHkg3w_JwEOeRgTYs0nD2AUZrR-Yd_cpN_DOCJXyX_uns-ksMU-FVw7cam75274XfI89rPJOAtg0wLIkdGWzVvxir0Nv0tL9Uq7UpzvJEDFWcV9TpOqn046-XCS8Hktv1_S9sXQ--yJQlNmqlC9FSAhzK3h4VXKoISxYwLuTSlqSw90uY0Zdfu-Fv3nerTsS3ayp6JqvhCCfP70FenuWZ".to_string());

    // Get Agent Details to Confirm Working
    match get_my_agent(&conf).await {
        Ok(res) => {
            println!("{:#?}", res);
            // Print Symbol
            println!("My Symbol: {:#?}", res.data.symbol);
            let document = to_document(&res.data)?;
            agent
                .replace_one(
                    doc! {"symbol": "TEST144121"},
                    document,
                    Some(ReplaceOptions::builder().upsert(true).build()),
                )
                .await?;
        }
        Err(err_res) => {
            panic!("{:#?}", err_res);
        }
    }

    // Get Data
    let result = agent.find_one(doc! {"symbol": "TEST144121"}, None).await?;

    match result {
        Some(ref document) => {
            let agent: spacedust::models::agent::Agent =
                from_bson(Bson::Document(document.to_owned()))?;
            println!("Found {:#?}", agent);
        }
        None => {
            println!("Could not find question for answer 42");
        }
    }

    Ok(())
}
