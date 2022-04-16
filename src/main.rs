use spacedust::client::Client;
use spacedust::shared;
// use std::collections::HashMap;
// use std::thread;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;

use tokio::time::{sleep, Duration};

mod cadets;
use crate::cadets::cadet;

mod duties;
use crate::duties::{Duty, DutyClass};

mod captains_log;
use crate::captains_log::{CaptainsLog, ShipWithCooldowns};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Setup Game Client
    let http_client = spacedust::client::get_http_client(None);

    // // Register agent
    // let claim_agent_response = spacedust::client::claim_agent(
    //     http_client,
    //     "https://v2-0-0.alpha.spacetraders.io".to_string(),
    //     "<4-8 character string>".to_string(),
    //     "COMMERCE_REPUBLIC".to_string(),
    // )
    // .await.unwrap();

    // // Setup client using claimed agent
    // let client = Client::new(
    //     http_client,
    //     "https://v2-0-0.alpha.spacetraders.io".to_string(),
    //     claim_agent_response.data.agent.symbol,
    //     claim_agent_response.token,
    // );

    let _client = Client::new(
        http_client,
        "https://v2-0-0.alpha.spacetraders.io".to_string(),
        "GREEN".to_string(),
        "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZGVudGlmaWVyIjoiR1JFRU4iLCJpYXQiOjE2NDcwMTY1NjMsInN1YiI6ImFnZW50LXRva2VuIn0.S-9AfG_asd21tsdGf9TF-cwML32x-TFd-b2n9WT21CKA3gkS9qhR15Zng9I2chv92NRriUGUDVb3flc-nZfnbDrMK_iBUbHT7oLiUu1X4Rr9HumsHUdSEltpVGxTvRm6-0udRgLuy9ndoXCxomUsTruszqdRZ5BJb9-2OYcP_kU6FnYcERDGoKNn6jISmPCaSnSs8nCDw5dbSrDF16mAJiGozJlx9j1gDUHWzeQZF7k4fonPxcLPGQjSa4mKIMaYYCh5oATW3wMh5qnXb-iz-wiwHZ7aXd1jkmDVQzeFXYqLpNf1jjQOXXdqEcZ_lFe79Mgeg1vuNtJDZpPNh-KC7P7YdC_F-7DYA82x6uYDPN8bwxcPd5uNmw0lZr5_C0lUI_z8-igPurxDBLizwjBdMdjIaqY2YSjEV_zocRy-I-N_0c43Dc9a5zZoFFH0DPwFrR2c9pp3tSkFsRMHp86SVlASIDXCQlgLvlNoDORi79dVR9ap64JgK3z-ttoJ_v90".to_string(),
    );

    ////////////////////////////////////////////////////////////////////////////

    let (cmd_tx, cmd_rx) = tokio::sync::watch::channel("run".to_string());
    let (duty_tx, duty_rx) = flume::bounded(32);
    println!("Spawn");
    let drx = duty_rx.clone();
    let crx = cmd_rx.clone();
    tokio::spawn(async move { cadet(drx, crx, "Redshirt".to_string()).await });

    // Sleep to allow the tokio processes to spawn
    sleep(Duration::from_millis(1000)).await;

    let ship: shared::Ship;
    match _client.get_my_ship("GREEN-1".to_string()).await {
        Ok(res) => {
            println!("ok {:#?}", res);
            ship = res.data;
        }
        Err(res_err) => {
            println!("err {:?}", res_err);
            return Ok(());
        }
    }

    // Setup ctrl-c handling
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // Initialize captains log
    let cl = Arc::new(Mutex::new(CaptainsLog::new()));
    let mut locked_cl = cl.lock().await;
    locked_cl.ships.push(ShipWithCooldowns {
        ship,
        navigation: None,
    });

    // Send duties to the cadets
    if locked_cl.ships[0].ship.location != None
        && locked_cl.ships[0].ship.location.as_ref().unwrap() != &"X1-OE-PM".to_string()
    {
        println!("tx travel to X1-OE-PM");
        drop(locked_cl);
        match duty_tx.send(Duty::new(
            DutyClass::Fly,
            "X1-OE-PM".to_string(),
            _client.clone(),
            cl.clone(),
        )) {
            Ok(_) => println!("tx travel to X1-OE-PM SENT"),
            Err(reserr) => println!("tx ERR {:#?}", reserr),
        }
    } else {
        println!("At X1-OE-PM or in transit, skipping travel");
        drop(locked_cl);
        running.store(false, Ordering::SeqCst);
    }

    let locked_cl = cl.lock().await;

    if locked_cl.ships[0].navigation.is_none()
        && locked_cl.ships[0].ship.location.as_ref().unwrap() != &"X1-OE-A005".to_string()
    {
        println!("tx travel to X1-OE-A005");
        drop(locked_cl);
        match duty_tx.send(Duty::new(
            DutyClass::Fly,
            "X1-OE-A005".to_string(),
            _client.clone(),
            cl.clone(),
        )) {
            Ok(_) => println!("tx travel to X1-OE-A005 SENT"),
            Err(reserr) => println!("tx ERR {:#?}", reserr),
        }
    } else {
        println!("At X1-OE-A005 or in transit, skipping travel");
        drop(locked_cl);
        running.store(false, Ordering::SeqCst);
    }

    while running.load(Ordering::SeqCst) {
        //
    }

    cmd_tx.send("shutdown".to_string()).unwrap();
    // Sleep to allow the processes to gracefully shutdown before killing process
    sleep(Duration::from_millis(5000)).await;

    //////////////////////////////////////////////////////////////////////////

    //////////////////////////////////////////////////////////////////////////
    // Jump
    // match _client.get_jump_cooldown("GREEN-1".to_string()).await {
    //     Ok(res) => {
    //         println!("ok {:#?}", res);
    //     }
    //     Err(res_err) => {
    //         println!("err {:?}", res_err);
    //     }
    // }
    // match _client
    //     .jump("GREEN-1".to_string(), "X1-EV".to_string())
    //     .await
    // {
    //     Ok(res) => {
    //         println!("ok {:#?}", res);
    //     }
    //     Err(res_err) => {
    //         println!("err {:?}", res_err);
    //     }
    // }

    ///////////////////////////////////////////////////////////////////////////

    // match client
    //     .navigate_ship("GREEN-1".to_string(), "X1-OE-A005".to_string())
    //     .await
    // {
    //     Ok(res) => {
    //         println!("ok {:#?}", res);
    //     }
    //     Err(res_err) => {
    //         println!("err {:?}", res_err);
    //     }
    // }
    // match client.get_my_ship("GREEN-1".to_string()).await {
    //     Ok(res) => {
    //         println!("ok {:#?}", res);
    //     }
    //     Err(res_err) => {
    //         println!("err {:?}", res_err);
    //     }
    // }
    // let mut surveys: Vec<spacedust::shared::Survey> = client
    //     .survey_surroundings("GREEN-1".to_string())
    //     .await
    //     .unwrap()
    //     .data
    //     .surveys;
    // surveys.sort_by(|a, b| b.expiration.cmp(&a.expiration));
    // let survey = &surveys[0];
    // let mut delay: Duration;
    // println!("Got surveys, choosing: {:#?}", survey);
    // loop {
    //     match client
    //         .extract_resources(
    //             "GREEN-1".to_string(),
    //             Some(spacedust::shared::Survey::clone(survey)),
    //         )
    //         .await
    //     {
    //         Ok(res) => {
    //             println!("ok {:#?}", res);
    //             delay = Duration::from_millis(res.data.cooldown.duration + 1);
    //         }
    //         Err(res_err) => {
    //             println!("err {:#?}", res_err);
    //             break;
    //         }
    //     }
    //     sleep(delay * 1000).await;
    // }
    // match client.get_my_contracts().await {
    //     Ok(res) => {
    //         println!("ok {:#?}", res);
    //     }
    //     Err(res_err) => {
    //         println!("err {:?}", res_err);
    //     }
    // }

    // match client
    //     .deliver_goods(
    //         "GREEN-1".to_string(),
    //         "cl0mn4xhw001401s6nt6m67rm".to_string(),
    //         "IRON_ORE".to_string(),
    //         1u64,
    //     )
    //     .await
    // {
    //     Ok(res) => {
    //         println!("ok {:#?}", res);
    //     }
    //     Err(res_err) => {
    //         println!("err {:?}", res_err);
    //     }
    // }

    ///////////////////////////////////////////////////////////////

    // match client.orbit_ship("GREEN-1".to_string()).await {
    //     Ok(res) => {
    //         println!("ok {:#?}", res);
    //     }
    //     Err(res_err) => {
    //         println!("err {:?}", res_err);
    //     }
    // }
    // match client
    //     .scan_ships("GREEN-1".to_string(), shared::ScanMode::ApproachingShips)
    //     .await
    // {
    //     Ok(res) => {
    //         println!("ok {:#?}", res);
    //     }
    //     Err(res_err) => {
    //         println!("err {:?}", res_err);
    //     }
    // }
    // match client.get_scan_cooldown("GREEN-1".to_string()).await {
    //     Ok(res) => {
    //         println!("ok {:#?}", res);
    //     }
    //     Err(res_err) => {
    //         println!("err {:?}", res_err);
    //     }
    // }
    // match client.dock_ship("GREEN-1".to_string()).await {
    //     Ok(res) => {
    //         println!("ok {:#?}", res);
    //     }
    //     Err(res_err) => {
    //         println!("err {:?}", res_err);
    //     }
    // }
    // match client
    //     .sell_cargo("GREEN-1".to_string(), "IRON_ORE".to_string(), 1u64)
    //     .await
    // {
    //     Ok(res) => {
    //         println!("ok {:#?}", res);
    //     }
    //     Err(res_err) => {
    //         println!("err {:?}", res_err);
    //     }
    // }
    // match client
    //     .buy_cargo("GREEN-1".to_string(), "MICROPROCESSORS".to_string(), 1u64)
    //     .await
    // {
    //     Ok(res) => {
    //         println!("ok {:#?}", res);
    //     }
    //     Err(res_err) => {
    //         println!("err {:?}", res_err);
    //     }
    // }
    // match client
    //     .jump("GREEN-1".to_string(), "X1-EV".to_string())
    //     .await
    // {
    //     Ok(res) => {
    //         println!("ok {:#?}", res);
    //     }
    //     Err(res_err) => {
    //         println!("err {:?}", res_err);
    //     }
    // }
    Ok(())
}
