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
use std::collections::HashMap;
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
use crate::log::{Log, LogSeverity};

mod steward;
use steward::Steward;

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

    // Get process status collection and steward
    let process_status_collection = log_db.collection("process_status");
    process_status_collection.drop(None).await.unwrap();
    let steward = Steward::new(process_status_collection);

    // Create MPMC log channel, used to route logs to the correct service
    let (log_tx, log_rx) = tokio::sync::mpsc::channel(1);

    // Get log collections for each level
    let mut log_collection_hashmap: HashMap<LogSeverity, Collection<Document>> = HashMap::new();
    let routine_log_collection = log_db.collection("routine");
    routine_log_collection.drop(None).await.unwrap();
    log_collection_hashmap.insert(LogSeverity::Routine, routine_log_collection);
    let priority_log_collection = log_db.collection("priority");
    priority_log_collection.drop(None).await.unwrap();
    log_collection_hashmap.insert(LogSeverity::Priority, priority_log_collection);
    let critical_log_collection = log_db.collection("critical");
    critical_log_collection.drop(None).await.unwrap();
    log_collection_hashmap.insert(LogSeverity::Critical, critical_log_collection);

    // Create log objects
    let mut log_object = Log::new(
        log_collection_hashmap.clone(),
        "LOG".to_string(),
        cmd_rx.clone(),
        log_rx,
    );
    // Set Process Status for Logs to STARTING
    steward.process_start("LOG".to_string()).await;

    // Set Pre-Spawn Timestamp
    let pre_log_spawn_timestamp = Utc::now();

    // Spawn threads for each log
    let log_steward = steward.clone();
    tokio::spawn(async move { log_object.initialize(log_steward).await });

    // Wait for Initialization Status to be Ready for Each in DB
    let mut log_process_state = "STARTING".to_string();

    while log_process_state == "READY".to_string() {
        print!(
            "\rWaiting for Log to be State READY: {} | Time Elapsed: {}",
            log_process_state,
            Utc::now()
                .signed_duration_since(pre_log_spawn_timestamp)
                .to_string()
        );

        // Check Statuses
        log_process_state = steward.get_process_state("LOG".to_string()).await;
    }
    print!(
        "\rWaiting for Log to be State READY: {} | Time Elapsed: {}\n",
        log_process_state,
        Utc::now()
            .signed_duration_since(pre_log_spawn_timestamp)
            .to_string()
    );
    println!("Log Finished Initializing at Startup");

    // Attempt Registration or Login
    // TODO: Implement Here

    // Get Agent Symbol
    let agent_symbol: String = "TESTAGENT".to_string();

    // Create Cadets
    let navigator = Navigator::new(
        "NAVIGATOR".to_string(),
        agent_symbol.to_string(),
        cmd_rx.clone(),
        log_tx.clone(),
    );
    let factor = Factor::new(
        "FACTOR".to_string(),
        agent_symbol.to_string(),
        cmd_rx.clone(),
        log_tx.clone(),
    );
    let astropath = Astropath::new(
        "ASTROPATH".to_string(),
        agent_symbol.to_string(),
        cmd_rx.clone(),
        log_tx.clone(),
    );
    let admiral = Admiral::new(
        "ADMIRAL".to_string(),
        agent_symbol.to_string(),
        cmd_rx.clone(),
        log_tx.clone(),
    );

    // Mark each cadet as STARTING in process_status
    steward
        .process_start(format!("{}::NAVIGATOR", agent_symbol))
        .await;
    steward
        .process_start(format!("{}::FACTOR", agent_symbol))
        .await;
    steward
        .process_start(format!("{}::ASTROPATH", agent_symbol))
        .await;
    steward
        .process_start(format!("{}::ADMIRAL", agent_symbol))
        .await;

    // Set Pre-Spawn Timestamp
    let pre_cadet_spawn_timestamp = Utc::now();

    // Spawn threads for each cadet
    let navigator_steward = steward.clone();
    tokio::spawn(async move { navigator.initialize(navigator_steward).await });
    let factor_steward = steward.clone();
    tokio::spawn(async move { factor.initialize(factor_steward).await });
    let astropath_steward = steward.clone();
    tokio::spawn(async move { astropath.initialize(astropath_steward).await });
    let admiral_steward = steward.clone();
    tokio::spawn(async move { admiral.initialize(admiral_steward).await });

    // Wait for Initialization Status to be Ready for Each in DB
    let mut cadet_processes_ready = false;
    let mut navigator_process_state = "STARTING".to_string();
    let mut factor_process_state = "STARTING".to_string();
    let mut astropath_process_state = "STARTING".to_string();
    let mut admiral_process_state = "STARTING".to_string();

    while cadet_processes_ready == false {
        print!(
            "\rWaiting for Cadets to be State READY | Navigator: {}, Factor: {}, Astropath: {}, Admiral: {} | Time Elapsed: {}",
            navigator_process_state,
            factor_process_state,
            astropath_process_state,
            admiral_process_state,
            Utc::now()
                .signed_duration_since(pre_cadet_spawn_timestamp)
                .to_string()
        );

        // Check Statuses
        navigator_process_state = steward
            .get_process_state(format!("{}::NAVIGATOR", agent_symbol))
            .await;
        factor_process_state = steward
            .get_process_state(format!("{}::FACTOR", agent_symbol))
            .await;
        astropath_process_state = steward
            .get_process_state(format!("{}::ASTROPATH", agent_symbol))
            .await;
        admiral_process_state = steward
            .get_process_state(format!("{}::ADMIRAL", agent_symbol))
            .await;

        cadet_processes_ready = navigator_process_state == "READY".to_string()
            && factor_process_state == "READY".to_string()
            && astropath_process_state == "READY".to_string()
            && admiral_process_state == "READY".to_string();
    }
    print!(
        "\rWaiting for Cadets to be State READY | Navigator: {}, Factor: {}, Astropath: {}, Admiral: {} | Time Elapsed: {}\n",
        navigator_process_state,
        factor_process_state,
        astropath_process_state,
        admiral_process_state,
        Utc::now()
            .signed_duration_since(pre_cadet_spawn_timestamp)
            .to_string()
    );

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
