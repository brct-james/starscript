use mongodb::{
    bson::{doc, to_document, Document},
    options::ReplaceOptions,
    Collection,
};

use crate::log::{ProcessState, ProcessStatus};

// Steward handles interactions with process_status collection
#[derive(Debug, Clone)]
pub struct Steward {
    process_status_table: Collection<Document>,
}

impl Steward {
    pub fn new(process_status_table: Collection<Document>) -> Self {
        Self {
            process_status_table,
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
                to_document(&ProcessStatus::new(pid.to_string(), ProcessState::STARTING)).unwrap(),
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
}
