use chrono::prelude::Utc;
use futures::FutureExt;
use mongodb::bson::{doc, to_document, DateTime, Document};
use mongodb::Collection;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver as MPSCReceiver;
use tokio::sync::watch::Receiver as SPMCReceiver;

use crate::safe_panic::safe_panic;
use crate::steward::{ProcessState, Steward};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessStatus {
    pub uuid: bson::Uuid,
    pub last_update_timestamp: DateTime,
    pub process_id: String,
    pub state: String,
}

impl ProcessStatus {
    pub fn new(process_id: String, state: ProcessState) -> Self {
        Self {
            uuid: bson::Uuid::new(),
            last_update_timestamp: Utc::now().into(),
            process_id,
            state: state.to_string(),
        }
    }
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
    table: Collection<Document>,
    label: String,
    cmd_rx: SPMCReceiver<String>,
    log_rx: MPSCReceiver<Message>,
}

impl Log {
    pub fn new(
        table: Collection<Document>,
        label: String,
        cmd_rx: SPMCReceiver<String>,
        log_rx: MPSCReceiver<Message>,
    ) -> Self {
        Self {
            table,
            label,
            cmd_rx,
            log_rx,
        }
    }

    pub async fn initialize(&mut self, steward: Steward) {
        // Log Initialized Message
        let process_id = format!("{}", self.label);

        let init_message = LogSchema::new(
            LogSeverity::Routine.to_string(),
            process_id.to_string(),
            format!(
                "Initializing {} - {}",
                process_id.to_string(),
                LogSeverity::Routine.to_string(),
            ),
        );
        let init_document = to_document(&init_message).unwrap();
        self.table.insert_one(init_document, None).await.unwrap();

        // Set Process Status
        steward.process_ready(process_id.to_string()).await;

        // Use select to follow the branch for if either cmd or msg received
        loop {
            futures::select! {
                _ = self.cmd_rx.changed().fuse() => {
                    let cmd = self.cmd_rx.borrow().to_string();
                    if cmd == String::from("shutdown") {
                        steward.process_stop(process_id.to_string()).await;
                        println!("Closed log {}", self.label);
                        return;
                    }
                },
                message = self.log_rx.recv().fuse() => {
                    match message {
                        Some(msg) => {
                            let log_msg = LogSchema::new(
                                msg.severity.to_string(),
                                msg.origin.to_string(),
                                msg.content.to_string(),
                            );

                            let document = to_document(&log_msg).unwrap();
                            self.table.insert_one(document, None).await.unwrap();
                        },
                        None => {
                            let log_msg = LogSchema::new(
                                LogSeverity::Critical.to_string(),
                                "LOG".to_string(),
                                "LOG TX DISCONNECTED".to_string(),
                            );

                            let document = to_document(&log_msg).unwrap();
                            self.table.insert_one(document, None).await.unwrap();

                            safe_panic("LOG TX DISCONNECTED".to_string(), &steward).await;
                        }
                    }
                },
            }
        }
    }
}
