use std::collections::HashMap;

use chrono::prelude::Utc;
use futures::FutureExt;
use mongodb::bson::{doc, to_document};
use mongodb::{bson::Document, Collection};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::Receiver as MPSCReceiver;
use tokio::sync::watch::Receiver as SPMCReceiver;

use crate::safe_panic::safe_panic;
use crate::steward::{ProcessState, Steward};

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
    log_to_severities: HashMap<LogSeverity, Vec<LogSeverity>>,
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
            log_to_severities: HashMap::from([
                (LogSeverity::Routine, vec![LogSeverity::Routine]),
                (
                    LogSeverity::Priority,
                    vec![LogSeverity::Routine, LogSeverity::Priority],
                ),
                (
                    LogSeverity::Critical,
                    vec![
                        LogSeverity::Routine,
                        LogSeverity::Priority,
                        LogSeverity::Critical,
                    ],
                ),
            ]),
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

        // Use try_select to watch for either a log message or a command
        // let try_select_result = futures::future::try_select(self.log_rx.recv().map(|l| -> Result<Message, String> {
        //     match l {
        //         Some(message) => Ok(message),
        //         None => Err("No Log Message".to_string()),
        //     }
        // }).boxed(), self.cmd_rx.changed().boxed()).await;

        // match try_select_result {
        //     Ok(either_ok) => {
        //         let res = futures::future::Either::into_inner(either_ok);
        //     },
        //     Err(either_err) => safe_panic("Error while awaiting log or command: {:#?}", e)
        // }

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

                            for severity in self.log_to_severities.get(&msg.severity).unwrap() {
                                let document = to_document(&log_msg).unwrap();
                                let table = self.tables.get(severity).unwrap();
                                table.insert_one(document, None).await.unwrap();
                            }
                        },
                        None => {
                            let log_msg = LogSchema::new(
                                LogSeverity::Critical.to_string(),
                                "LOG".to_string(),
                                "LOG TX DISCONNECTED".to_string(),
                            );

                            for severity in self.log_to_severities.get(&LogSeverity::Critical).unwrap() {
                                let document = to_document(&log_msg).unwrap();
                                let table = self.tables.get(severity).unwrap();
                                table.insert_one(document, None).await.unwrap();
                            }

                            safe_panic("LOG TX DISCONNECTED".to_string(), &steward).await;
                        }
                    }
                },
            }
        }
    }
}
