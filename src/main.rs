// Uncategorized Imports
// use std::sync::atomic::{AtomicBool, Ordering};
// use std::sync::Arc;
// use tokio::sync::Mutex;

use mongodb::bson::{doc, Document};
use mongodb::{options::ClientOptions, Client};
use mongodb::{Collection, Database};

use dotenv;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use http::status::StatusCode;

use chrono::prelude::Utc;

use std::sync::atomic::{AtomicBool, Ordering};

// API
use spacedust::apis::agents_api::get_my_agent;
use spacedust::apis::default_api::register;
use spacedust::models::register_request::{Faction, RegisterRequest};

// Modules
// mod cadets;

// mod duties;

// mod captains_log;

// mod astronomicon;

// mod signaller;

mod cadet;
use cadet::{admiral::Admiral, astropath::Astropath, factor::Factor, navigator::Navigator};

mod log;
use crate::dbqueue::{Task, TaskPriority};
use crate::log::{Log, LogSeverity, Message};

mod steward;
use steward::Steward;

mod identity_manager;
use identity_manager::{AgentIdentity, IdentityManager};

mod safe_panic;
use safe_panic::safe_panic;

mod dbqueue;
use dbqueue::DBQueue;

mod rules;
use rules::RulesManager;

#[tokio::main]
async fn main() {
    // Setup ctrl-c handling
    let quit = Arc::new(AtomicBool::new(false));
    let q = quit.clone();
    ctrlc::set_handler(move || {
        q.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // Get Mongo Creds from Env
    dotenv::from_filename("mongo_secrets.env").ok();

    // Parse a mongo connection string into an options struct
    let connection_string = format!(
        "mongodb://{}:{}@localhost:27018",
        env::var("MONGO_INITDB_ROOT_USERNAME").unwrap(),
        env::var("MONGO_INITDB_ROOT_PASSWORD").unwrap()
    );
    let mut client_options = ClientOptions::parse(connection_string).await.unwrap();

    // Set client app name
    client_options.app_name = Some("starscript-client".to_string());

    // Get a handle to the deployment.
    let client = Client::with_options(client_options).unwrap();

    // DEPRECATED: Get a handle to each database
    // let db = client.database("starscript");

    // Create SPMC command channel, used to gracefully shutdown threads
    let (cmd_tx, cmd_rx) = tokio::sync::watch::channel("run".to_string());
    cmd_tx.send("run".to_string()).unwrap();

    // Run Setup Logging
    let log_tx_max_size: usize = 1;
    let (log_tx, steward) = setup_logging(&client, cmd_tx, &cmd_rx, log_tx_max_size).await;
    println!("Log Finished Initializing at Startup");

    // Run Handle Agent Login and Registration
    let logged_in_agents = setup_identities(&client, &log_tx, &steward).await;

    // Create collections for game data
    let game_db = client.database("starscript-gamedata");
    let gamedata_collections = get_collections(
        game_db,
        HashMap::from([
            ("astronomicon".to_string(), true),
            ("fleet".to_string(), true),
        ]),
    )
    .await;

    // // Create collections for settings
    // let settings_db = client.database("starscript-settings");
    // let settings_collections = get_collections(
    //     settings_db,
    //     HashMap::from([
    //             ("staleness".to_string(), true),
    //     ]),
    // )
    // .await;

    // Create rules manager and get staleness rules
    let rules_manager = RulesManager::new();
    let staleness_rules = rules_manager.get_staleness_rules();

    // Create collection for cadet queues
    let queue_db = client.database("starscript-queues");
    let cadet_queue_collections = get_collections(
        queue_db,
        HashMap::from([
            ("admiral".to_string(), true),
            ("astropath".to_string(), true),
            ("factor".to_string(), true),
            ("navigator".to_string(), true),
            ("ensign".to_string(), true),
        ]),
    )
    .await;

    // Create dbqueue managers for cadets
    let admiral_queue = DBQueue::new(cadet_queue_collections.clone(), "admiral".to_string());
    let astropath_queue = DBQueue::new(cadet_queue_collections.clone(), "astropath".to_string());
    let factor_queue = DBQueue::new(cadet_queue_collections.clone(), "factor".to_string());
    let navigator_queue = DBQueue::new(cadet_queue_collections.clone(), "navigator".to_string());
    let _ensign_queue = DBQueue::new(cadet_queue_collections.clone(), "ensign".to_string());

    // Create Cadets
    println!("Starting Cadet Creation");
    for (agent_symbol, agent_api_config) in logged_in_agents {
        println!("Creating Cadets for Agent {}", agent_symbol);
        // Create Cadets
        let mut navigator = Navigator::new(
            "NAVIGATOR".to_string(),
            agent_symbol.to_string(),
            cmd_rx.clone(),
            log_tx.clone(),
            navigator_queue.clone(),
        );
        let mut factor = Factor::new(
            "FACTOR".to_string(),
            agent_symbol.to_string(),
            cmd_rx.clone(),
            log_tx.clone(),
            factor_queue.clone(),
        );
        let mut astropath = Astropath::new(
            "ASTROPATH".to_string(),
            agent_symbol.to_string(),
            cmd_rx.clone(),
            log_tx.clone(),
            astropath_queue.clone(),
            agent_api_config.clone(),
            gamedata_collections.clone(),
            staleness_rules.clone(),
        );
        let mut admiral = Admiral::new(
            "ADMIRAL".to_string(),
            agent_symbol.to_string(),
            cmd_rx.clone(),
            log_tx.clone(),
            admiral_queue.clone(),
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

        println!("Processes Marked as Start for each Cadet, now Spawning for Initialization");
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
        let mut navigator_process_state: String;
        let mut factor_process_state: String;
        let mut astropath_process_state: String;
        let mut admiral_process_state: String;

        println!("Processes spawned, polling process_status table to check whether cadets are initialized");
        while cadet_processes_ready == false {
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

            println!(
                "Waiting for Cadets to be State READY | Navigator: {}, Factor: {}, Astropath: {}, Admiral: {} | Time Elapsed: {}s",
                navigator_process_state,
                factor_process_state,
                astropath_process_state,
                admiral_process_state,
                Utc::now()
                    .signed_duration_since(pre_cadet_spawn_timestamp)
                    .num_seconds()
            );

            sleep(Duration::from_millis(100)).await;
        }

        println!(
            "All Non-Ensign Cadets Initialized for Agent {}",
            agent_symbol
        );
    }

    println!("All Non-Ensign Cadets Created");

    println!("Enqueing DB Refresh from API");
    astropath_queue
        .send_task(Task::new(
            TaskPriority::ASAP,
            "refresh_systems".to_string(),
            HashMap::new(),
            None,
        ))
        .await;
    astropath_queue
        .send_task(Task::new(
            TaskPriority::ASAP,
            "refresh_fleet".to_string(),
            HashMap::new(),
            None,
        ))
        .await;
    astropath_queue
        .send_task(Task::new(
            TaskPriority::ASAP,
            "refresh_fleet".to_string(),
            HashMap::new(),
            None,
        ))
        .await;

    println!("Initialization Finished, Entering Game Loop, Hit Ctrl+C to exit");
    loop {
        // println!("------- At Beginning of Loop -------");
        // CHECK IF NEED TO QUIT AND IF SO GRACEFULLY SHUTDOWN
        if quit.load(Ordering::SeqCst) {
            // cmd_tx.send("shutdown".to_string()).unwrap();
            // // Sleep to allow the processes to gracefully shutdown before killing process
            // // TODO: Use a channel to confirm shutdown instead
            // sleep(Duration::from_millis(5000)).await;
            break;
        }
        sleep(Duration::from_millis(100)).await;
    }

    safe_panic("Finished Main".to_string(), &steward).await;
}

async fn setup_identities(
    client: &Client,
    log_tx: &tokio::sync::mpsc::Sender<Message>,
    steward: &Steward,
) -> HashMap<String, spacedust::apis::configuration::Configuration> {
    let identity_db = client.database("starscript-identities");

    let identity_collections =
        get_collections(identity_db, HashMap::from([("agents".to_string(), true)])).await;

    // identity_collections.get("agents").unwrap().clone()
    let identity_manager =
        IdentityManager::new(identity_collections.get("agents").unwrap().clone());
    identity_manager.save_agent("test144121".to_string(), "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZGVudGlmaWVyIjoiVEVTVDE0NDEyMSIsImlhdCI6MTY3NzM1MzkzNywic3ViIjoiYWdlbnQtdG9rZW4ifQ.sKH_plunZQl8qbu_wR1a65beysxCpYrHwmZwXadlT9rzkHhxXcDf7WczbOFHjD9mjas88lL2k_BJWJ5GCfn3HAoQCTIjHA3IbSh7ugHptjVBByRFSRzV2yddTZgFKKgc7nD7128w23w7t2mYAZ9q-1m8fSwMqrKPP7_RgDMtrt6nOW3VHyS5aPSssiu8bsdDqMzUAkEwy1N2SYVZTIOQL6IEq5OYljY9QONy2isXkJSGvWlFGNlZH6WQAYQeXKc5fvIUv5rG4hH0oGjeb6ncnyZUQjEmIlDGoRWsiXWgUS0iHkg3w_JwEOeRgTYs0nD2AUZrR-Yd_cpN_DOCJXyX_uns-ksMU-FVw7cam75274XfI89rPJOAtg0wLIkdGWzVvxir0Nv0tL9Uq7UpzvJEDFWcV9TpOqn046-XCS8Hktv1_S9sXQ--yJQlNmqlC9FSAhzK3h4VXKoISxYwLuTSlqSw90uY0Zdfu-Fv3nerTsS3ayp6JqvhCCfP70FenuWZ".to_string()).await;

    // Look for agents.json defining symbols to load
    let agent_symbols: Vec<String>;
    let load_agents_result = load_agents_json();
    match load_agents_result {
        Ok(agents) => {
            agent_symbols = agents.active.clone();
        }
        _ => panic!("Could not load json/agents.json"),
    }

    if agent_symbols.len() < 1 {
        panic!("No agents loaded from json/agents.json")
    }

    let mut agents_to_login: Vec<AgentIdentity> = Default::default();
    let mut agents_to_register: Vec<String> = Default::default();
    for symbol in agent_symbols {
        let get_agent_result = identity_manager.get_agent(symbol.to_string()).await;
        match get_agent_result {
            Some(aid) => agents_to_login.push(aid),
            None => agents_to_register.push(symbol.to_string()),
        }
    }

    // Attempt to Login agents
    let mut logged_in_agents: HashMap<String, spacedust::apis::configuration::Configuration> =
        Default::default();
    for id in agents_to_login {
        // Create API Configuration
        let mut conf = spacedust::apis::configuration::Configuration::new();
        conf.bearer_access_token = Some(id.get_token());
        match get_my_agent(&conf).await {
            Ok(res) => {
                logged_in_agents.insert(id.get_symbol(), conf.clone());
                log_tx
                    .send(Message::new(
                        LogSeverity::Routine,
                        "MAIN".to_string(),
                        format!("Logged in Agent {}", res.data.symbol),
                    ))
                    .await
                    .unwrap();
            }
            Err(spacedust::apis::Error::ResponseError(e)) => match e.status {
                StatusCode::UNAUTHORIZED => {
                    agents_to_register.push(id.get_symbol());
                    log_tx
                            .send(Message::new(
                                LogSeverity::Routine,
                                "MAIN".to_string(),
                                format!("Failed to Login Agent {}, Attempting Registration Instead, Reason: Unauthorized (Invalid Token?): {:?}", id.get_symbol(), e),
                            ))
                            .await
                            .unwrap();
                }
                _ => {
                    agents_to_register.push(id.get_symbol());
                    log_tx
                            .send(Message::new(
                                LogSeverity::Routine,
                                "MAIN".to_string(),
                                format!("Failed to Login Agent {}, Attempting Registration Instead, Reason: Uncaught Reason: {:?}", id.get_symbol(), e),
                            ))
                            .await
                            .unwrap();
                }
            },
            Err(e) => {
                log_tx
                    .send(Message::new(
                        LogSeverity::Priority,
                        "MAIN".to_string(),
                        format!("Failed to Login Agent {}, Attempting Registration Instead, Reason: No or Invalid Response Received: {:?}", id.get_symbol(), e),
                    ))
                    .await
                    .unwrap();
            }
        }
    }

    // Attempt To Register Agents
    for symbol in agents_to_register {
        // Create Register Request
        let reg_req = RegisterRequest::new(Faction::Cosmic, symbol.to_string());

        // Register Agent
        let mut conf = spacedust::apis::configuration::Configuration::new();
        let register_response = register(&conf, Some(reg_req)).await;

        match register_response {
            Ok(res) => {
                // Update Config with Agent Token
                conf.bearer_access_token = Some(res.data.token);
                logged_in_agents.insert(symbol.to_string(), conf.clone());
                log_tx
                    .send(Message::new(
                        LogSeverity::Routine,
                        "MAIN".to_string(),
                        format!("Registered Agent {}", symbol.to_string()),
                    ))
                    .await
                    .unwrap();
            }
            Err(spacedust::apis::Error::ResponseError(e)) => match e.status {
                StatusCode::UNPROCESSABLE_ENTITY => {
                    log_tx
                            .send(Message::new(
                                LogSeverity::Routine,
                                "MAIN".to_string(),
                                format!("Failed to Register Agent {}, Reason: Unprocessable (Symbol Already Taken or Invalid Symbol?): {:?}", symbol.to_string(), e),
                            ))
                            .await
                            .unwrap();
                }
                _ => {
                    log_tx
                        .send(Message::new(
                            LogSeverity::Priority,
                            "MAIN".to_string(),
                            format!(
                                "Failed to Register Agent {}, Reason: Uncaught Reason: {:?}",
                                symbol.to_string(),
                                e
                            ),
                        ))
                        .await
                        .unwrap();
                }
            },
            Err(e) => {
                log_tx
                    .send(Message::new(
                        LogSeverity::Priority,
                        "MAIN".to_string(),
                        format!("Failed to Register Agent {}, Reason: No or Invalid Response Received: {:?}", symbol.to_string(), e),
                    ))
                    .await
                    .unwrap();
            }
        }
    }

    if logged_in_agents.len() < 1 {
        safe_panic("No agents successfully logged in OR registered, check priority log for more information".to_string(), steward).await;
    }

    return logged_in_agents;
}

async fn setup_logging(
    client: &Client,
    cmd_tx: tokio::sync::watch::Sender<String>,
    cmd_rx: &tokio::sync::watch::Receiver<String>,
    log_tx_max_size: usize,
) -> (tokio::sync::mpsc::Sender<Message>, Steward) {
    let log_db = client.database("starscript-logs");
    let (log_tx, log_rx) = tokio::sync::mpsc::channel(log_tx_max_size);

    // Initialize Log DB
    let log_collections = get_collections(
        log_db,
        HashMap::from([
            ("process_status".to_string(), true),
            ("log".to_string(), true),
        ]),
    )
    .await;

    // Create Process Steward
    let steward = Steward::new(
        log_collections.get("process_status").unwrap().clone(),
        Arc::new(cmd_tx),
    );

    // Create log object
    let mut log_object = Log::new(
        log_collections.get("log").unwrap().clone(),
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
        // Check Statuses
        log_process_state = steward.get_process_state("LOG".to_string()).await;

        sleep(Duration::from_millis(1000)).await;
        print!(
            "Waiting for Log to be State READY: {} | Time Elapsed: {}s",
            log_process_state,
            Utc::now()
                .signed_duration_since(pre_log_spawn_timestamp)
                .num_seconds()
        );
    }

    return (log_tx.clone(), steward.clone());
}

async fn get_collections(
    db: Database,
    cname_drop: HashMap<String, bool>,
) -> HashMap<String, Collection<Document>> {
    let mut res: HashMap<String, Collection<Document>> = Default::default();
    for (cname, drop_first) in cname_drop {
        let collection = db.collection(cname.as_str());
        if drop_first {
            collection.drop(None).await.unwrap();
        }
        res.insert(cname, collection);
    }
    return res;
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct AgentsJSON {
    active: Vec<String>,
}

impl AgentsJSON {
    fn new() -> Self {
        Self {
            active: Default::default(),
        }
    }
}

fn load_agents_json() -> Result<AgentsJSON, std::io::Error> {
    let filename = "json/agents.json";
    let f = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .read(true)
        .open(filename)?;
    // serialize json as HashMap
    match serde_json::from_reader(f) {
        Ok(stats) => Ok(stats),
        Err(e) if e.is_eof() => Ok(AgentsJSON::new()),
        Err(e) => panic!("An error occurred: {}", e),
    }
}
