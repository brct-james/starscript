use crate::log::{LogSeverity, Message};
use tokio::sync::broadcast::Sender as BroadcastSender;
use tokio::sync::watch::Receiver as SPMCREceiver;

pub struct Admiral {
    id: String,
    rank: String,
    agent_symbol: String,
    cmd_rx: SPMCREceiver<String>,
    log_tx: BroadcastSender<Message>,
}

impl Admiral {
    pub fn new(
        id: String,
        agent_symbol: String,
        cmd_rx: SPMCREceiver<String>,
        log_tx: BroadcastSender<Message>,
    ) -> Self {
        Self {
            id,
            rank: "Admiral".to_string(),
            agent_symbol,
            cmd_rx,
            log_tx,
        }
    }

    pub async fn initialize(&self) {
        self.log_tx
            .send(Message::new(
                LogSeverity::Routine,
                format!("{}: {} - {}", self.agent_symbol, self.rank, self.id),
                format!(
                    "Initializing {} for agent {} with ID {} ",
                    self.rank, self.agent_symbol, self.id
                ),
            ))
            .unwrap();
        let mut cmd = "run".to_string();
        while cmd == "run".to_string() {
            cmd = self.cmd_rx.borrow().to_string();
        }
        self.log_tx
            .send(Message::new(
                LogSeverity::Routine,
                format!("{}: {} - {}", self.agent_symbol, self.rank, self.id),
                format!("Closed {} with ID {}", self.rank, self.id),
            ))
            .unwrap();
    }
}
