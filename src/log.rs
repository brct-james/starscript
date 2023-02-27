use chrono::prelude::Utc;
use mongodb::bson::{doc, to_document};
use mongodb::{bson::Document, Collection};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast::error::TryRecvError;
use tokio::sync::broadcast::Receiver as BroadcastReceiver;
use tokio::sync::watch::Receiver as SPMCReceiver;

use crate::steward::Steward;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ProcessStatus {
    pub last_update_timestamp: String,
    pub process_id: String,
    pub state: String,
}

impl ProcessStatus {
    pub fn new(process_id: String, state: ProcessState) -> Self {
        Self {
            last_update_timestamp: Utc::now().to_string(),
            process_id,
            state: state.to_string(),
        }
    }
}

#[derive(strum_macros::Display, Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub enum ProcessState {
    #[default]
    STARTING,
    READY,
    CLOSED,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Message {
    pub severity: LogSeverity,
    pub origin: String,
    pub content: String,
}

impl Message {
    pub fn new(severity: LogSeverity, origin: String, content: String) -> Self {
        Self {
            severity,
            origin,
            content,
        }
    }
}

#[derive(strum_macros::Display, Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub enum LogSeverity {
    #[default]
    Routine,
    Priority,
    Critical,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct LogSchema {
    timestamp: String,
    severity: String,
    origin: String,
    content: String,
}

impl LogSchema {
    pub fn new(severity: String, origin: String, content: String) -> Self {
        Self {
            timestamp: Utc::now().to_string(),
            severity,
            origin,
            content,
        }
    }
}

pub struct Log {
    table: Collection<Document>,
    label: String,
    severities: Vec<LogSeverity>,
    cmd_rx: SPMCReceiver<String>,
    log_rx: BroadcastReceiver<Message>,
}

impl Log {
    pub fn new(
        table: Collection<Document>,
        label: String,
        severities: Vec<LogSeverity>,
        cmd_rx: SPMCReceiver<String>,
        log_rx: BroadcastReceiver<Message>,
    ) -> Self {
        Self {
            table,
            label,
            severities,
            cmd_rx,
            log_rx,
        }
    }

    pub async fn initialize(&mut self, steward: Steward) {
        // Log Initialized Message
        let process_id = format!("LOG::{}", self.label);
        let init_message = LogSchema::new(
            LogSeverity::Critical.to_string(),
            "LOG".to_string(),
            format!(
                "Initializing '{}' with severities {:?}",
                process_id.to_string(),
                self.format_severities(&self.severities)
            ),
        );
        let init_document = to_document(&init_message).unwrap();
        self.table.insert_one(init_document, None).await.unwrap();

        // Set Process Status
        steward.process_ready(process_id.to_string()).await;

        let mut cmd = "run".to_string();
        while cmd == "run".to_string() {
            cmd = self.cmd_rx.borrow().to_string();
            let recv = self.log_rx.try_recv();
            match recv {
                Ok(msg) => {
                    println!(
                        "\n{} MESSAGE FROM {}, SEVERITY: {:?} IN {:?}?{}\n",
                        process_id,
                        msg.origin,
                        msg.severity,
                        self.severities,
                        self.severities.contains(&msg.severity)
                    );
                    let message = LogSchema::new(
                        "TEST".to_string(),
                        msg.origin.to_string(),
                        msg.content.to_string(),
                    );
                    let document = to_document(&message).unwrap();
                    self.table.insert_one(document, None).await.unwrap();
                    if self.severities.contains(&msg.severity) {
                        let message = LogSchema::new(
                            msg.severity.to_string(),
                            msg.origin.to_string(),
                            msg.content.to_string(),
                        );
                        let document = to_document(&message).unwrap();
                        self.table.insert_one(document, None).await.unwrap();
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Closed) => panic!("LOG TX CLOSED"),
                Err(e) => println!("LOG LAGGED: {}", e),
            }
        }
        steward.process_stop(process_id.to_string()).await;
        println!("Closed log {}", self.label);
    }

    fn format_severities(&self, severities: &Vec<LogSeverity>) -> Vec<String> {
        let mut res: Vec<String> = Default::default();
        for sev in severities {
            res.push(sev.to_string());
        }
        return res;
    }
}
