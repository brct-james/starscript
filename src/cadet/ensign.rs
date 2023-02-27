use crate::log::{LogSeverity, Message};
use crate::steward::Steward;
use tokio::sync::mpsc::Sender as MPSCSender;
use tokio::sync::watch::Receiver as SPMCREceiver;

pub struct Ensign {
    label: String,
    rank: String,
    agent_symbol: String,
    cmd_rx: SPMCREceiver<String>,
    log_tx: MPSCSender<Message>,
    ship_symbol: String,
}

impl Ensign {
    pub fn new(
        label: String,
        agent_symbol: String,
        cmd_rx: SPMCREceiver<String>,
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

    pub async fn initialize(&self, steward: Steward) {
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
        let mut cmd = "run".to_string();
        while cmd == "run".to_string() {
            cmd = self.cmd_rx.borrow().to_string();
        }
        steward.process_stop(process_id.to_string()).await;
        self.log_tx
            .send(Message::new(
                LogSeverity::Routine,
                process_id.to_string(),
                format!(
                    "Closed {} with ID {} for ship {}",
                    self.rank, self.label, self.ship_symbol
                ),
            ))
            .await
            .unwrap();
    }
}
