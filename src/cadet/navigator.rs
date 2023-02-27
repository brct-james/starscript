use crate::cadet::Cadet;
use crate::log::{LogSeverity, Message};
use flume::Sender;
use tokio::sync::watch::Receiver;

pub struct Navigator {
    id: String,
    rank: String,
    agent_symbol: String,
    cmd_rx: Receiver<String>,
    log_tx: Sender<Message>,
}

impl Navigator {
    pub fn new(
        id: String,
        agent_symbol: String,
        cmd_rx: Receiver<String>,
        log_tx: Sender<Message>,
    ) -> Self {
        Self {
            id,
            rank: "Navigator".to_string(),
            agent_symbol,
            cmd_rx,
            log_tx,
        }
    }
}

impl Cadet for Navigator {
    fn initialize(&self) {
        self.log_tx.send(Message::new(
            LogSeverity::Routine,
            format!("{}: {} - {}", self.agent_symbol, self.rank, self.id),
            format!(
                "Initializing {} for agent {} with ID {} ",
                self.rank, self.agent_symbol, self.id
            ),
        ));
        let mut cmd = "run".to_string();
        while cmd == "run".to_string() {
            cmd = self.cmd_rx.borrow().to_string();
        }
        self.log_tx.send(Message::new(
            LogSeverity::Routine,
            format!("{}: {} - {}", self.agent_symbol, self.rank, self.id),
            format!("Closed {} with ID {}", self.rank, self.id),
        ));
    }
}
