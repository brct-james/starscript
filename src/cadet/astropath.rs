use crate::dbqueue::{DBQueue, Task};
use crate::log::{LogSeverity, Message};
use crate::rules::StalenessRules;
use crate::safe_panic::safe_panic;
use crate::steward::Steward;
use bson::{doc, from_bson, to_document, Bson, Document};
use chrono::Utc;
use futures::FutureExt;
use http::StatusCode;
use mongodb::bson::DateTime;
use mongodb::options::ReplaceOptions;
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use spacedust::apis::fleet_api::get_my_ships;
use spacedust::apis::Error::ResponseError;
use spacedust::models::Ship;
use std::collections::HashMap;
use tokio::sync::mpsc::Sender as MPSCSender;
use tokio::sync::watch::Receiver as SPMCReceiver;
use tokio::time::{sleep, Duration};

pub struct Astropath {
    label: String,
    rank: String,
    agent_symbol: String,
    cmd_rx: SPMCReceiver<String>,
    log_tx: MPSCSender<Message>,
    queue: DBQueue,
    api_config: spacedust::apis::configuration::Configuration,
    data_tables: HashMap<String, Collection<Document>>,
    staleness_rules: StalenessRules,
    refresh_timestamps: HashMap<String, DateTime>,
}

impl Astropath {
    pub fn new(
        label: String,
        agent_symbol: String,
        cmd_rx: SPMCReceiver<String>,
        log_tx: MPSCSender<Message>,
        queue: DBQueue,
        api_config: spacedust::apis::configuration::Configuration,
        data_tables: HashMap<String, Collection<Document>>,
        staleness_rules: StalenessRules,
    ) -> Self {
        Self {
            label,
            rank: "Astropath".to_string(),
            agent_symbol,
            cmd_rx,
            log_tx,
            queue,
            api_config,
            data_tables,
            staleness_rules,
            refresh_timestamps: HashMap::<String, DateTime>::new(),
        }
    }

    pub async fn initialize(&mut self, steward: Steward) {
        let process_id = format!("{}::{}", self.agent_symbol, self.label);
        self.log_tx
            .send(Message::new(
                LogSeverity::Routine,
                process_id.to_string(),
                format!(
                    "Initializing {} for agent {} with ID {} ",
                    self.rank, self.agent_symbol, self.label
                ),
            ))
            .await
            .unwrap();
        steward.process_ready(process_id.to_string()).await;

        // Use select to follow the branch for if either cmd or msg received
        loop {
            futures::select! {
                task_option = self.queue.get_task().fuse() => {
                    match task_option {
                        Some(task) => {
                            let task_cmd = task.command.to_string();
                            self.log_tx.send(Message::new(LogSeverity::Routine, process_id.to_string(), format!("Got Task from Queue: {}", task_cmd))).await.unwrap();
                            if task_cmd == String::from("refresh_systems") {
                                self.refresh_systems(process_id.to_string(), task, &steward).await;
                            } else if task_cmd == String::from("refresh_fleet") {
                                self.refresh_fleet(process_id.to_string(), task, &steward).await;
                            } else {
                                self.log_tx.send(Message::new(LogSeverity::Critical, process_id.to_string(), format!("Received unhandled task command: {}", task_cmd))).await.unwrap();
                                safe_panic(format!("Received unhandled task command: {}", task_cmd), &steward).await;
                            }
                        },
                        None => sleep(Duration::from_millis(10)).await,
                    }
                },
                _ = self.cmd_rx.changed().fuse() => {
                    let cmd = self.cmd_rx.borrow().to_string();
                    if cmd == String::from("shutdown") {
                        steward.process_stop(process_id.to_string()).await;
                        // println!("Closed cadet {}", self.label);
                        return;
                    }
                },
            }
        }
    }

    async fn is_data_stale(
        &self,
        process_id: &String,
        rule_name: &String,
        steward: &Steward,
    ) -> bool {
        let rule_option = self.staleness_rules.get(rule_name);
        match rule_option {
            Some(rule) => {
                let refresh_timestamp_option = self.refresh_timestamps.get(rule_name);

                let last: i64;
                match refresh_timestamp_option {
                    Some(refresh_timestamp) => {
                        last = refresh_timestamp.timestamp_millis();
                    }
                    None => {
                        last = 0;
                    }
                }

                let now = Utc::now().timestamp_millis();
                let millis_since_refresh = now - last;

                if rule.clone() <= millis_since_refresh {
                    self.log_tx
                        .send(Message::new(
                            LogSeverity::Routine,
                            process_id.to_string(),
                            format!(
                                "IS_DATA_STALE: Data corresponding to Rule {} IS stale ({} <= {})",
                                rule_name, rule, millis_since_refresh
                            ),
                        ))
                        .await
                        .unwrap();
                    return true;
                }
                self.log_tx
                    .send(Message::new(
                        LogSeverity::Routine,
                        process_id.to_string(),
                        format!(
                            "IS_DATA_STALE: Data corresponding to Rule {} IS NOT stale ({} !<= {})",
                            rule_name, rule, millis_since_refresh
                        ),
                    ))
                    .await
                    .unwrap();
                return false;
            }
            None => {
                safe_panic(
                    format!("Could not get staleness rule for name {}", rule_name),
                    steward,
                )
                .await;
                return true;
            }
        }
    }

    async fn refresh_systems(&mut self, process_id: String, task: Task, steward: &Steward) {
        let rule_name = "systems".to_string();
        if self.is_data_stale(&process_id, &rule_name, &steward).await {
            self.log_tx
                .send(Message::new(
                    LogSeverity::Routine,
                    process_id.to_string(),
                    format!("REFRESH_SYSTEMS: Requesting Full Systems File"),
                ))
                .await
                .unwrap();

            // TODO: Once systems.json PR is merged, push update to cargo, then use it here
            sleep(Duration::from_millis(1000)).await;
            // // TODO: Handle instead of panic
            // safe_panic(
            //     "Could not refresh_systems, see priority log for more details".to_string(),
            //     &steward,
            // )
            // .await;

            // SUCCESS
            self.refresh_timestamps.insert(
                rule_name,
                DateTime::from_millis(Utc::now().timestamp_millis()),
            );
        } else {
            self.log_tx
                .send(Message::new(
                    LogSeverity::Routine,
                    process_id.to_string(),
                    format!("REFRESH_SYSTEMS: No need to refresh systems because data not stale",),
                ))
                .await
                .unwrap();
        }
        match &task.callback {
            Some(builder) => {
                let cb_task = builder.build();
                self.queue
                    .send_task_any_queue(builder.get_queue_name(), cb_task)
                    .await;
                self.log_tx
                    .send(Message::new(
                        LogSeverity::Routine,
                        process_id.to_string(),
                        format!(
                            "REFRESH_SYSTEMS: Queued callback with {}",
                            builder.get_queue_name(),
                        ),
                    ))
                    .await
                    .unwrap();
            }
            None => (),
        }
        self.queue.finish_task(task).await;
    }

    async fn refresh_fleet(&mut self, process_id: String, task: Task, steward: &Steward) {
        let rule_name = "fleet".to_string();
        if self.is_data_stale(&process_id, &rule_name, &steward).await {
            self.log_tx
                .send(Message::new(
                    LogSeverity::Routine,
                    process_id.to_string(),
                    format!("REFRESH_FLEET: Requesting Full Fleet Update"),
                ))
                .await
                .unwrap();

            let mut fleet: Vec<Ship> = Default::default();
            let limit: i32 = 20;
            let mut page: i32 = 1;
            let mut count: i32 = 0;
            let mut total: i32 = 1;

            while count < total {
                match get_my_ships(&self.api_config, Some(page), Some(limit)).await {
                    Ok(response) => {
                        total = response.meta.limit;
                        count += limit;
                        page += 1;
                        fleet.extend(response.data.into_iter());
                    }
                    Err(ResponseError(e)) => match e.status {
                        StatusCode::TOO_MANY_REQUESTS => {
                            // TODO: Make a PR in the docs repo to generate error handling for ratelimit and other error codes instead of rolling my own ratelimit error object
                            println!("rl_Content: {:#?}", &e.content);
                            let rl_err: RatelimitErrorWrapper =
                                serde_json::from_str(&e.content).unwrap();
                            sleep(Duration::from_millis(rl_err.get_retry_after_millis() + 1)).await;
                        }
                        _ => {
                            self.log_tx
                                .send(Message::new(
                                    LogSeverity::Priority,
                                    "ASTROPATH".to_string(),
                                    format!("REFRESH_FLEET: Failed to get_my_ships, ResponseError, reason {:?}", e),
                                ))
                                .await
                                .unwrap();
                            safe_panic(
                                "Could not refresh_fleet, see priority log for more details"
                                    .to_string(),
                                &steward,
                            )
                            .await;
                            return;
                        }
                    },
                    Err(e) => {
                        self.log_tx
                            .send(Message::new(
                                LogSeverity::Priority,
                                "ASTROPATH".to_string(),
                                format!("REFRESH_FLEET: Failed to get_my_ships, reason {:?}", e),
                            ))
                            .await
                            .unwrap();
                        safe_panic(
                            "Could not refresh_fleet, see priority log for more details"
                                .to_string(),
                            &steward,
                        )
                        .await;
                        return;
                    }
                }
            }
            self.log_tx
                .send(Message::new(
                    LogSeverity::Routine,
                    process_id.to_string(),
                    format!("REFRESH_FLEET: Received All Ships from API, Storing..."),
                ))
                .await
                .unwrap();

            let fleet_table = self.data_tables.get("fleet").unwrap();
            for ship in fleet.iter() {
                let mut replacement_ship = StoredShip::new(ship.clone());

                // Check for existing ship in DB, persist UUID if found
                let get_result = fleet_table
                    .find_one(doc! {"symbol": ship.symbol.to_string()}, None)
                    .await
                    .unwrap();
                match get_result {
                    Some(record) => {
                        let stored_ship: StoredShip = from_bson(Bson::Document(record)).unwrap();
                        replacement_ship.update_uuid(&stored_ship.uuid);
                    }
                    None => (),
                }

                fleet_table
                    .replace_one(
                        doc! {"symbol": ship.symbol.to_string()},
                        to_document(&replacement_ship).unwrap(),
                        Some(ReplaceOptions::builder().upsert(true).build()),
                    )
                    .await
                    .unwrap();
            }

            self.log_tx
                .send(Message::new(
                    LogSeverity::Routine,
                    process_id.to_string(),
                    format!(
                        "REFRESH_FLEET: Finished storing ships, total stored: {}",
                        fleet.len()
                    ),
                ))
                .await
                .unwrap();

            // SUCCESS
            self.refresh_timestamps.insert(
                rule_name,
                DateTime::from_millis(Utc::now().timestamp_millis()),
            );
        } else {
            self.log_tx
                .send(Message::new(
                    LogSeverity::Routine,
                    process_id.to_string(),
                    format!("REFRESH_FLEET: No need to refresh fleet because data not stale",),
                ))
                .await
                .unwrap();
        }
        match &task.callback {
            Some(builder) => {
                let cb_task = builder.build();
                self.queue
                    .send_task_any_queue(builder.get_queue_name(), cb_task)
                    .await;
                self.log_tx
                    .send(Message::new(
                        LogSeverity::Routine,
                        process_id.to_string(),
                        format!(
                            "REFRESH_FLEET: Queued callback with {}",
                            builder.get_queue_name(),
                        ),
                    ))
                    .await
                    .unwrap();
            }
            None => (),
        }
        self.queue.finish_task(task).await;
    }
}

// /// Defines a structure for retaining information on the galaxy, its systems, and waypoints
// #[derive(Serialize, Deserialize, Debug, Clone, Default)]
// pub struct Astronomicon {
//     pub systems: String,
//     pub waypoints: HashMap<String, String>,
//     pub last_updated: DateTime,
// }

// impl Astronomicon {
//     pub fn new() -> Self {
//         Astronomicon::default()
//     }
// }

/// Defines a structure for ships stored in the DB
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StoredShip {
    uuid: bson::Uuid,
    symbol: String,
    data: Ship,
    last_update_timestamp: DateTime,
}

impl StoredShip {
    pub fn new(data: Ship) -> Self {
        Self {
            uuid: bson::Uuid::new(),
            symbol: data.symbol.to_string(),
            data,
            last_update_timestamp: Utc::now().into(),
        }
    }

    pub fn update_uuid(&mut self, new_uuid: &bson::Uuid) {
        self.uuid = new_uuid.clone();
    }
}

/// Defines a structure for the wrapper around ratelimit error responses
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct RatelimitErrorWrapper {
    error: RatelimitError,
}

impl RatelimitErrorWrapper {
    pub fn get_retry_after_millis(&self) -> u64 {
        (self.error.data.retry_after * 1000.0).ceil() as u64
    }
}

/// Defines a structure for ratelimit error responses
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct RatelimitError {
    message: String,
    code: i32,
    data: RatelimitErrorData,
}

/// Defines a structure for ratelimit error data field
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct RatelimitErrorData {
    #[serde(rename = "type")]
    timeout_type: String,
    #[serde(rename = "retryAfter")]
    retry_after: f32,
    #[serde(rename = "limitBurst")]
    limit_burst: i32,
    #[serde(rename = "limitPerSecond")]
    limit_per_second: i32,
    remaining: i32,
    #[serde(rename = "reset")]
    reset_timestamp: String,
}
