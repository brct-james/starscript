use crate::log::{LogSeverity, Message};
use crate::steward::Steward;
use tokio::sync::mpsc::Sender as MPSCSender;
use tokio::sync::watch::Receiver as SPMCREceiver;

pub struct Astropath {
    label: String,
    rank: String,
    agent_symbol: String,
    cmd_rx: SPMCREceiver<String>,
    log_tx: MPSCSender<Message>,
}

impl Astropath {
    pub fn new(
        label: String,
        agent_symbol: String,
        cmd_rx: SPMCREceiver<String>,
        log_tx: MPSCSender<Message>,
    ) -> Self {
        Self {
            label,
            rank: "Astropath".to_string(),
            agent_symbol,
            cmd_rx,
            log_tx,
        }
    }

    pub async fn initialize(&self, steward: Steward) {
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
        let mut cmd = "run".to_string();
        while cmd == "run".to_string() {
            cmd = self.cmd_rx.borrow().to_string();
        }
        steward.process_stop(process_id.to_string()).await;
        self.log_tx
            .send(Message::new(
                LogSeverity::Routine,
                process_id.to_string(),
                format!("Closed {} with ID {}", self.rank, self.label),
            ))
            .await
            .unwrap();
    }
}
