use chrono::Utc;
use tokio::time::{sleep, Duration};

use crate::steward::Steward;

pub async fn safe_panic(panic: String, steward: &Steward) {
    println!("SAFE PANIC CALLED");
    let shutdown_timestamp = Utc::now();
    steward.safe_shutdown();

    let mut processes_remain = true;
    while processes_remain {
        let processes_remaining = steward.check_shutdown_status().await;
        print!(
            "\rWaiting for processes to shutdown gracefully. Time Elapsed: {}s | Remaining: {:?}",
            Utc::now()
                .signed_duration_since(shutdown_timestamp)
                .num_seconds(),
            processes_remaining,
        );
        if processes_remaining.len() == 0 {
            processes_remain = false;
        }
        sleep(Duration::from_millis(100)).await;
    }
    print!(
        "\rWaiting for processes to shutdown gracefully. Elapsed: {}s\n",
        Utc::now()
            .signed_duration_since(shutdown_timestamp)
            .num_seconds()
    );
    sleep(Duration::from_millis(1000)).await;
    panic!("{}", panic);
}