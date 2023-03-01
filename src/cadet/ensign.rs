use crate::log::{LogSeverity, Message};
use crate::steward::Steward;
use futures::FutureExt;
use tokio::sync::mpsc::Sender as MPSCSender;
use tokio::sync::watch::Receiver as SPMCReceiver;

#[allow(dead_code)]
pub struct Ensign {
    label: String,
    rank: String,
    agent_symbol: String,
    cmd_rx: SPMCReceiver<String>,
    log_tx: MPSCSender<Message>,
    ship_symbol: String,
}

#[allow(dead_code)]
impl Ensign {
    pub fn new(
        label: String,
        agent_symbol: String,
        cmd_rx: SPMCReceiver<String>,
        log_tx: MPSCSender<Message>,
        ship_symbol: String,
    ) -> Self {
        Self {
            label,
            rank: "Ensign".to_string(),
            agent_symbol,
            cmd_rx,
            log_tx,
            ship_symbol,
        }
    }

    pub async fn initialize(&mut self, steward: Steward) {
        let process_id = format!(
            "{}::{}::{}",
            self.agent_symbol, self.label, self.ship_symbol
        );
        self.log_tx
            .send(Message::new(
                LogSeverity::Routine,
                process_id.to_string(),
                format!(
                    "Initializing {} for agent {} with ID {} for ship {}",
                    self.rank, self.agent_symbol, self.label, self.ship_symbol
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
