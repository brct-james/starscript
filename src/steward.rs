use futures::stream::StreamExt;
use std::sync::Arc;

use mongodb::{
    bson::{doc, from_bson, to_document, Bson, Document},
    options::ReplaceOptions,
    Collection,
};
use serde::{Deserialize, Serialize};

use crate::log::ProcessStatus;

// Describes the state of processes
#[derive(strum_macros::Display, Serialize, Deserialize, Debug, Clone, Default, Eq, PartialEq)]
pub enum ProcessState {
    #[default]
    START,
    READY,
    CLOSE,
}

// Steward handles interactions with process_status collection
#[derive(Debug, Clone)]
pub struct Steward {
    process_status_table: Collection<Document>,
    cmd_tx: Arc<tokio::sync::watch::Sender<String>>,
}

impl Steward {
    pub fn new(
        process_status_table: Collection<Document>,
        cmd_tx: Arc<tokio::sync::watch::Sender<String>>,
    ) -> Self {
        Self {
            process_status_table,
            cmd_tx,
        }
    }

    // Returns string for process state, or NOT_FOUND if not found
    pub async fn get_process_state(&self, pid: String) -> String {
        let found = self
            .process_status_table
            .find_one(Some(doc! {"process_id": pid}), None)
            .await
            .unwrap();

        match found {
            Some(ref document) => return document.get_str("state").unwrap().to_string(),
            None => return "NOT_FOUND".to_string(),
        }
    }

    pub async fn process_start(&self, pid: String) {
        self.process_status_table
            .replace_one(
                doc! {"process_id": pid.to_string()},
                to_document(&ProcessStatus::new(pid.to_string(), ProcessState::START)).unwrap(),
                Some(ReplaceOptions::builder().upsert(true).build()),
            )
            .await
            .unwrap();
    }

    pub async fn process_ready(&self, pid: String) {
        self.process_status_table
            .replace_one(
                doc! {"process_id": pid.to_string()},
                to_document(&ProcessStatus::new(pid.to_string(), ProcessState::READY)).unwrap(),
                Some(ReplaceOptions::builder().upsert(true).build()),
            )
            .await
            .unwrap();
    }

    pub async fn process_stop(&self, pid: String) {
        self.process_status_table
            .replace_one(
                doc! {"process_id": pid.to_string()},
                to_document(&ProcessStatus::new(pid.to_string(), ProcessState::CLOSED)).unwrap(),
                Some(ReplaceOptions::builder().upsert(true).build()),
            )
            .await
            .unwrap();
    }

    pub fn safe_shutdown(&self) {
        self.cmd_tx.send("shutdown".to_string()).unwrap();
    }

    pub async fn check_shutdown_status(&self) -> Vec<ProcessStatus> {
        let filtered_cursor = self
            .process_status_table
            .find(
                Some(doc! {"state": { "$ne": ProcessState::CLOSED.to_string() }}),
                None,
            )
            .await
            .unwrap();

        let filtered_vec: Vec<Result<Document, mongodb::error::Error>> =
            filtered_cursor.collect().await;

        let formatted_vec: Vec<ProcessStatus> = filtered_vec
            .into_iter()
            .map(|rd| from_bson(Bson::Document(rd.unwrap())).unwrap())
            .collect();
        return formatted_vec;
    }
}
