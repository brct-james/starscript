use std::collections::HashMap;

use chrono::prelude::Utc;
use mongodb::bson::{doc, to_document};
use mongodb::{bson::Document, Collection};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::mpsc::Receiver as MPSCReceiver;
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

#[derive(
    strum_macros::Display, Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq, Hash,
)]
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
    tables: HashMap<LogSeverity, Collection<Document>>,
    label: String,
    cmd_rx: SPMCReceiver<String>,
    log_rx: MPSCReceiver<Message>,
}

impl Log {
    pub fn new(
        tables: HashMap<LogSeverity, Collection<Document>>,
        label: String,
        cmd_rx: SPMCReceiver<String>,
        log_rx: MPSCReceiver<Message>,
    ) -> Self {
        Self {
            tables,
            label,
            cmd_rx,
            log_rx,
        }
    }

    pub async fn initialize(&mut self, steward: Steward) {
        // Log Initialized Message
        let process_id = format!("{}", self.label);

        for (severity, table) in self.tables.iter() {
            let init_message = LogSchema::new(
                severity.to_string(),
                process_id.to_string(),
                format!(
                    "Initializing {} - {}",
                    process_id.to_string(),
                    severity.to_string(),
                ),
            );
            let init_document = to_document(&init_message).unwrap();
            table.insert_one(init_document, None).await.unwrap();
        }

        // Set Process Status
        steward.process_ready(process_id.to_string()).await;

        let mut cmd = "run".to_string();
        while cmd == "run".to_string() {
            cmd = self.cmd_rx.borrow().to_string();
            let recv = self.log_rx.try_recv();
            match recv {
                Ok(msg) => {
                    let log_to_severities: Vec<LogSeverity>;
                    match msg.severity {
                        LogSeverity::Routine => {
                            log_to_severities = vec![LogSeverity::Routine];
                        }
                        LogSeverity::Priority => {
                            log_to_severities = vec![LogSeverity::Routine, LogSeverity::Priority];
                        }
                        LogSeverity::Critical => {
                            log_to_severities = vec![
                                LogSeverity::Routine,
                                LogSeverity::Priority,
                                LogSeverity::Critical,
                            ];
                        }
                    }

                    for severity in log_to_severities {
                        let table = self.tables.get(&severity).unwrap();
                        let message = LogSchema::new(
                            msg.severity.to_string(),
                            msg.origin.to_string(),
                            msg.content.to_string(),
                        );
                        let document = to_document(&message).unwrap();
                        table.insert_one(document, None).await.unwrap();
                    }
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => panic!("LOG TX DISCONNECTED"),
            }
        }
        steward.process_stop(process_id.to_string()).await;
        println!("Closed log {}", self.label);
    }
}
