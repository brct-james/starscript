use crate::log::{LogSeverity, Message};
use tokio::sync::broadcast::Sender as BroadcastSender;
use tokio::sync::watch::Receiver as SPMCREceiver;

pub struct Ensign {
    id: String,
    rank: String,
    agent_symbol: String,
    cmd_rx: SPMCREceiver<String>,
    log_tx: BroadcastSender<Message>,
    ship_symbol: String,
}

impl Ensign {
    pub fn new(
        id: String,
        agent_symbol: String,
        cmd_rx: SPMCREceiver<String>,
        log_tx: BroadcastSender<Message>,
        ship_symbol: String,
    ) -> Self {
        Self {
            id,
            rank: "Ensign".to_string(),
            agent_symbol,
            cmd_rx,
            log_tx,
            ship_symbol,
        }
    }

    pub async fn initialize(&self) {
        self.log_tx
            .send(Message::new(
                LogSeverity::Routine,
                format!(
                    "{} ({}): {} - {}",
                    self.agent_symbol, self.ship_symbol, self.rank, self.id
                ),
                format!(
                    "Initializing {} for agent {} and ship {} with ID {} ",
                    self.rank, self.agent_symbol, self.ship_symbol, self.id
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
                format!(
                    "{} ({}): {} - {}",
                    self.agent_symbol, self.ship_symbol, self.rank, self.id
                ),
                format!("Closed {} with ID {}", self.rank, self.id),
            ))
            .unwrap();
    }
}
