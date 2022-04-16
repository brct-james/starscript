use super::captains_log::CaptainsLog;
use spacedust::client::Client;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct Duty {
    pub class: DutyClass,
    pub command: String,
    pub client: Client,
    pub state: Arc<Mutex<CaptainsLog>>,
}

impl Duty {
    pub fn new(
        class: DutyClass,
        command: String,
        client: Client,
        state: Arc<Mutex<CaptainsLog>>,
    ) -> Self {
        Duty {
            class,
            command,
            client,
            state,
        }
    }

    pub async fn execute(&self) -> String {
        let mut data = self.state.lock().await;

        let cur_loc = data.ships[0].ship.location.clone().unwrap();

        let status: String;

        match self
            .client
            .navigate_ship("GREEN-1".to_string(), self.command.to_string())
            .await
        {
            Ok(res) => {
                println!("ok {:#?}", res);
                data.ships[0].navigation = Some(res.data.navigation.clone());
                status = format!(
                    "Success, arrive in: {:#?}",
                    res.data.navigation.duration_remaining.unwrap()
                )
            }
            Err(res_err) => {
                println!("err {:?}", res_err);
                status = "Failure".to_string();
            }
        }

        format!(
            "{:#?}: {:#?} -> {}: {}",
            self.class, cur_loc, self.command, status,
        )
    }
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub enum DutyClass {
    Fly,
}

// pub async fn duty(
//     duty_rx: flume::Receiver<i32>,
//     cmd_rx: tokio::sync::watch::Receiver<String>,
//     id: String,
// ) {
//     let mut cmd = cmd_rx.borrow().to_string();
//     println!("{} INIT CMD: {}", id, cmd);
//     while cmd == "run" {
//         let i_res = duty_rx.try_recv();
//         cmd = cmd_rx.borrow().to_string();
//         match i_res {
//             Ok(i) => println!("{} ok: {}", id, i),
//             Err(flume::TryRecvError::Empty) => (),
//             Err(_) => cmd = "err".to_string(),
//         }
//     }
//     println!("{} DONE: {}", id, cmd);
// }
