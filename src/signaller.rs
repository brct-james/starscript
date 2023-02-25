use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A signaller instance is responsible for queueing and synchronously processing requests sent by cadets, and handling ratelimiting
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Signaller {
    pub queue: Vec<String>,
}

impl Signaller {
    pub fn new() -> Self {
        Signaller::default()
    }

    pub fn listen(msg_rx: flume::Receiver<String>, cmd_rx: tokio::sync::watch::Receiver<String>, reply_tx: tokio::sync::watch::Sender<String>) {
        let mut cmd = cmd_rx.borrow().to_string();
        while cmd == "run" {
            let recv_res = msg_rx.try_recv();
            cmd = cmd_rx.borrow().to_string();
            match recv_res {
                Ok(msg) => println!("{}", msg),
                Err(flume::TryRecvError::Empty) => (),
                Err(_) => cmd = "err".to_string(),
            }
        }
        println!("DONE: {}", cmd);
    }

    pub fn send(&mut self, uuid: String) {
        self.queue.push(uuid);
    }
}