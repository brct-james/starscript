use crate::log::{LogSeverity, Message};
use crate::steward::Steward;
use futures::FutureExt;
use tokio::sync::mpsc::Sender as MPSCSender;
use tokio::sync::watch::Receiver as SPMCReceiver;

pub struct Navigator {
    label: String,
    rank: String,
    agent_symbol: String,
    cmd_rx: SPMCReceiver<String>,
    log_tx: MPSCSender<Message>,
}

impl Navigator {
    pub fn new(
        label: String,
        agent_symbol: String,
        cmd_rx: SPMCReceiver<String>,
        log_tx: MPSCSender<Message>,
    ) -> Self {
        Self {
            label,
            rank: "Navigator".to_string(),
            agent_symbol,
            cmd_rx,
            log_tx,
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
}
