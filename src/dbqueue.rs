use futures::stream::StreamExt;
use std::collections::HashMap;

use bson::{doc, from_bson, to_document, Bson};
use chrono::Utc;
use mongodb::{bson::DateTime, bson::Document, options::FindOptions, Collection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct DBQueue {
    queue_collections: HashMap<String, Collection<Document>>,
    queue_name: String,
}

#[allow(dead_code)]
impl DBQueue {
    pub fn new(
        queue_collections: HashMap<String, Collection<Document>>,
        queue_name: String,
    ) -> Self {
        Self {
            queue_collections,
            queue_name,
        }
    }

    pub async fn get_task(&self) -> Option<Task> {
        let queue_table = self.queue_collections.get(&self.queue_name).unwrap();
        let found_cursor = queue_table
            .find(
                None,
                FindOptions::builder()
                    .limit(1)
                    .sort(doc! { "priority": 1, "timestamp": -1 })
                    .build(),
            )
            .await
            .unwrap();

        let found_vec: Vec<Result<Document, mongodb::error::Error>> = found_cursor.collect().await;

        let formatted_vec: Vec<Task> = found_vec
            .into_iter()
            .map(|rd| from_bson(Bson::Document(rd.unwrap())).unwrap())
            .collect();

        if formatted_vec.len() == 0 {
            return None;
        }

        let found = formatted_vec[0].clone();

        return Some(found);
    }

    pub async fn send_task_any_queue(&self, queue_name: String, task: Task) {
        let queue_table = self.queue_collections.get(&queue_name).unwrap();
        queue_table
            .insert_one(to_document(&task).unwrap(), None)
            .await
            .unwrap();
    }

    pub async fn send_task(&self, task: Task) {
        self.send_task_any_queue(self.queue_name.to_string(), task)
            .await;
    }

    pub async fn send_callback(&self, builder: CallbackTaskBuilder) {
        let target = builder.get_queue_name();
        self.send_task_any_queue(target, builder.build()).await;
    }

    async fn delete_task(&self, task: Task) {
        let queue_table = self.queue_collections.get(&self.queue_name).unwrap();
        queue_table
            .delete_one(to_document(&task).unwrap(), None)
            .await
            .unwrap();
    }

    pub async fn finish_task(&self, task: Task) {
        match task.clone().callback {
            Some(cb) => {
                self.send_callback(cb).await;
                self.delete_task(task).await;
            }
            None => self.delete_task(task).await,
        }
    }
}

// Contains the necessary information to handle returning data to another process after one is processed from the queue
// Callbacks should be constructed such that they require no data to be passed outside the DB
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct CallbackTaskBuilder {
    queue_name: String,
    priority: TaskPriority,
    command: String,
    parameters: HashMap<String, String>,
}

#[allow(dead_code)]
impl CallbackTaskBuilder {
    pub fn new(
        queue_name: String,
        priority: TaskPriority,
        command: String,
        parameters: HashMap<String, String>,
    ) -> Self {
        Self {
            queue_name,
            priority,
            command,
            parameters,
        }
    }

    fn get_queue_name(&self) -> String {
        self.queue_name.to_string()
    }

    fn build(&self) -> Task {
        Task::new(
            self.priority.clone(),
            self.command.to_string(),
            self.parameters.clone(),
            None,
        )
    }
}

// Enum for task priority
#[derive(
    strum_macros::Display,
    Serialize,
    Deserialize,
    Debug,
    Clone,
    Default,
    Eq,
    PartialEq,
    Hash,
    Ord,
    PartialOrd,
)]
pub enum TaskPriority {
    Low = 4,
    #[default]
    Normal = 3,
    High = 2,
    ASAP = 1,
}

// The representation of queue tasks in the DB
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub uuid: bson::Uuid,
    pub priority: TaskPriority,
    pub timestamp: DateTime,
    pub command: String,
    pub parameters: HashMap<String, String>,
    pub callback: Option<CallbackTaskBuilder>,
}

impl Task {
    pub fn new(
        priority: TaskPriority,
        command: String,
        parameters: HashMap<String, String>,
        callback: Option<CallbackTaskBuilder>,
    ) -> Self {
        Self {
            uuid: bson::Uuid::new(),
            priority,
            timestamp: Utc::now().into(),
            command,
            parameters,
            callback,
        }
    }
}
