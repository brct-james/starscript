use crate::duties::Duty;

pub async fn cadet(
    duty_rx: flume::Receiver<Duty>,
    cmd_rx: tokio::sync::watch::Receiver<String>,
    id: String,
) {
    let mut cmd = cmd_rx.borrow().to_string();
    println!("{} INIT CMD: {}", id, cmd);
    while cmd == "run" {
        let recv_res = duty_rx.try_recv();
        cmd = cmd_rx.borrow().to_string();
        match recv_res {
            Ok(duty) => println!("{}", duty.execute().await),
            Err(flume::TryRecvError::Empty) => (),
            Err(_) => cmd = "err".to_string(),
        }
    }
    println!("{} DONE: {}", id, cmd);
}
