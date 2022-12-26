use spacedust::apis::contracts_api::{accept_contract, deliver_contract, fulfill_contract};
use spacedust::apis::fleet_api::{
    create_survey, extract_resources, jettison, navigate_ship, refuel_ship,
};
use spacedust::apis::{configuration, contracts_api as contracts, fleet_api as fleet};
use spacedust::models::{
    DeliverContractRequest, ExtractResourcesRequest, JettisonRequest, NavigateShipRequest,
    ShipNavStatus,
};
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
use crate::captains_log::shiprole::ShipRole;
use crate::captains_log::CaptainsLog;

mod astronomicon;
use crate::astronomicon::Astronomicon;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Setup Configuration
    let mut client = configuration::Configuration::new();
    client.bearer_access_token = Some(String::from("eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZGVudGlmaWVyIjoiVklSSURJUyIsImlhdCI6MTY3MDY5MDY2Niwic3ViIjoiYWdlbnQtdG9rZW4ifQ.A42hIDXm2UV3VJGX8juWXUgXt3yU_x23VoPiZfJJapN4ogEjz9dEK9njzYI2FDIvQx6CliX7RHNbzkE1R98LBOjzkH-9eHPd_MkqLtu4-58n7YKNTaelzTRSCeIz4W-cnqwdUSvQCs8F4yzTt7UOCuJSJErY242xtGjVv72RdHY7prPU09PY79q0dwJ_utEi7Mu-CF96qGfD6sJ9gy8YXeiuNb4Kcm03H7QnqalFeZ-dCfxHQRWInzSv6OQrDjqXNUnhQLnWAxm0oqZTc0YbFNBoKsiVMOf8Qv-5Dw4rTB5GYkL81lLDyxKA9ijt2Lw_jhXB97xeCOnA7kud-mTcLXHaB_xVGTFdcyT-hDkjWMgEL1dQQmHcyRvBjenXrfGqNribVN0qdwyPwdgs9SU7u3BSFEvmI5gvJPcbI8TF6C8QXuIwHW6BjB_ELL6ZDFSBmbBnCp-55ILONbjKScBk4cV3cyCixPvVFJFGL_0padXvRM_nXgCuQcANLdH495Gh"));

    ////////////////////////////////////////////////////////////////////////////

    // let (cmd_tx, cmd_rx) = tokio::sync::watch::channel("run".to_string());
    // let (duty_tx, duty_rx) = flume::bounded(32);
    // println!("Spawn");
    // let drx = duty_rx.clone();
    // let crx = cmd_rx.clone();
    // tokio::spawn(async move { cadet(drx, crx, "Redshirt".to_string()).await });

    // // Sleep to allow the tokio processes to spawn
    // sleep(Duration::from_millis(1000)).await;

    // Setup ctrl-c handling
    let quit = Arc::new(AtomicBool::new(false));
    let q = quit.clone();
    ctrlc::set_handler(move || {
        q.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // Get Ship
    let ship = fleet::get_my_ship(&client, "VIRIDIS-1").await.unwrap().data;

    // Initialize captains log
    let cl = Arc::new(Mutex::new(CaptainsLog::new()));
    let mut locked_cl = cl.lock().await;
    locked_cl.upsert_ship(ship);

    // Initialize astronomicon
    // TODO: generate from game query
    let astr = Arc::new(Mutex::new(Astronomicon::new()));
    let mut locked_astr = astr.lock().await;
    // locked_cl.waypoints.push();

    // Send duties to the cadets
    println!("Start Loop");
    loop {
        println!("------- At Beginning of Loop -------");
        // CHECK IF NEED TO QUIT AND IF SO GRACEFULLY SHUTDOWN
        if quit.load(Ordering::SeqCst) {
            // cmd_tx.send("shutdown".to_string()).unwrap();
            // // Sleep to allow the processes to gracefully shutdown before killing process
            // // TODO: Use a channel to confirm shutdown instead
            // sleep(Duration::from_millis(5000)).await;
            break;
        }

        // Update fleet
        println!("Update Fleet");
        locked_cl.upsert_fleet(fleet::get_my_ships(&client).await.unwrap().data);

        // Update contracts
        println!("Update Contracts");
        let my_contracts = contracts::get_contracts(&client).await.unwrap().data;
        let mut active_contract: Option<spacedust::models::Contract> = None;

        // Accept Any That Haven't Been Accepted, Try Fulfill if Possible, Set an Active Contract
        for contract in my_contracts {
            if contract.accepted == false {
                accept_contract(&client, contract.id.as_str())
                    .await
                    .unwrap();
            } else if contract.fulfilled == false {
                let mut all_delivered = true;
                for delivery_term in contract.terms.deliver.clone().unwrap() {
                    if delivery_term.units_fulfilled < delivery_term.units_required {
                        all_delivered = false;
                        active_contract = Some(contract.clone());
                    }
                }
                if all_delivered {
                    fulfill_contract(&client, contract.id.as_str())
                        .await
                        .unwrap();
                }
            }
        }

        let mut active_contract_delivery_items: Vec<String> = Default::default();
        match active_contract {
            Some(ac) => {
                for item in ac.terms.deliver.unwrap() {
                    if item.units_fulfilled < item.units_required {
                        active_contract_delivery_items.push(item.trade_symbol);
                    }
                }
            }
            None => (),
        }

        locked_cl.upsert_ledger_contracts(contracts::get_contracts(&client).await.unwrap().data);

        for (symbol, ship) in locked_cl.ships.clone() {
            // If ship is unassigned, assign a role
            // TODO: Make this more intelligent, using some kind of global configuration to orchestrate goals broadly and the system will handle the logistics (e.g. 'complete this contract' and will send ships to mine and/or buy and turn in)
            if ship.role == ShipRole::Unassigned {
                println!("Assigning Ship {} Role::Miner", ship.ship.symbol);
                locked_cl.update_ship_role(&symbol, ShipRole::Miner);
                continue;
            }

            // If in transit, skip
            if ship.ship.nav.status == ShipNavStatus::InTransit {
                println!("Ship {} in transit, skipping", ship.ship.symbol);
                continue;
            }

            // TODO: Make cadet handle this logic internally based on role
            // E.g. if global config says priority is finishing a contract, based on the configured max distance for contract delivery ships will decide whether to sell their goods, or deliver.

            // TEMP: If miner, conduct mining
            let mining_waypoint = "X1-YR6-17535B";
            let deliver_waypoint = "X1-YR6-27710Z".to_string();
            if ship.role == ShipRole::Miner {
                println!("Ship {} is miner", ship.ship.symbol);
                if ship.ship.nav.waypoint_symbol == mining_waypoint {
                    println!("Ship {} at mining site", ship.ship.symbol);
                    // IF AT MINING SITE
                    if ship.ship.cargo.units >= ship.ship.cargo.capacity - 4 {
                        println!("Ship {} nearly full", ship.ship.symbol);
                        // IF CARGO WITHIN 4 UNITS OF FULL
                        // GO DELIVER OR SELL
                        // TODO: Sell logic
                        let mut did_jettison = false;
                        for item in ship.ship.cargo.inventory {
                            if active_contract_delivery_items.contains(&item.symbol) {
                                println!(
                                    "Ship {} has cargo to jettison: {} {}",
                                    ship.ship.symbol, item.units, item.symbol
                                );
                                did_jettison = true;
                                jettison(
                                    &client,
                                    ship.ship.symbol.as_str(),
                                    Some(JettisonRequest {
                                        symbol: item.symbol,
                                        units: item.units,
                                    }),
                                )
                                .await
                                .unwrap();
                            }
                        }
                        if !did_jettison {
                            println!(
                                "Ship {} didn't jettison, navigate to deliver waypoint",
                                ship.ship.symbol
                            );
                            navigate_ship(
                                &client,
                                ship.ship.symbol.as_str(),
                                Some(NavigateShipRequest {
                                    waypoint_symbol: deliver_waypoint,
                                }),
                            )
                            .await
                            .unwrap();
                        }
                    } else if locked_cl.ledger.survey_data.contains_key(mining_waypoint) {
                        // IF CARGO NOT FULL AND SURVEYS WERE AT ONE POINT ACTIVE FOR WAYPOINT
                        println!(
                            "Ship {} not full filtering most_recent_surveys",
                            ship.ship.symbol
                        );
                        // Filter expired surveys out of most_recent_surveys
                        locked_cl
                            .ledger
                            .survey_data
                            .get_mut(mining_waypoint)
                            .unwrap()
                            .most_recent_surveys = locked_cl
                            .ledger
                            .survey_data
                            .get(mining_waypoint)
                            .unwrap()
                            .most_recent_surveys
                            .clone()
                            .into_iter()
                            .filter(|surv| {
                                chrono::DateTime::parse_from_rfc3339(surv.expiration.as_str())
                                    .unwrap()
                                    > chrono::Utc::now()
                            })
                            .collect();

                        if locked_cl.ledger.survey_data[mining_waypoint]
                            .most_recent_surveys
                            .len()
                            > 0
                        {
                            // IF ACTIVE SURVEYS
                            println!("Ship {} has active surveys, mining", ship.ship.symbol);
                            // MINE WITH SURVEY
                            let target_survey = locked_cl.ledger.survey_data[mining_waypoint]
                                .most_recent_surveys[0]
                                .clone();
                            extract_resources(
                                &client,
                                ship.ship.symbol.as_str(),
                                Some(ExtractResourcesRequest {
                                    survey: Some(Box::new(target_survey)),
                                }),
                            )
                            .await
                            .unwrap();
                            continue;
                        } else {
                            // IF NO ACTIVE SURVEYS
                            println!("Ship {} no active surveys", ship.ship.symbol);
                            if ship.ship.cargo.units as f64 / ship.ship.cargo.capacity as f64 > 0.90
                                || ship.ship.cargo.capacity - ship.ship.cargo.units < 20
                            {
                                // IF CARGO NEARLY FULL
                                println!(
                                    "Ship {} cargo nearly full, mine without survey",
                                    ship.ship.symbol
                                );
                                // MINE WITHOUT SURVEY
                                // If ship at least 90% full or less than 20 units remaining just mine without survey
                                extract_resources(&client, ship.ship.symbol.as_str(), None)
                                    .await
                                    .unwrap();
                                continue;
                            } else {
                                // IF CARGO NOT NEARLY FULL
                                println!("Ship {} try survey", ship.ship.symbol);
                                // TRY SURVEY
                                let survey_result =
                                    create_survey(&client, ship.ship.symbol.as_str()).await;
                                match survey_result {
                                    Ok(survey_response) => {
                                        // IF SURVEY SUCCESS
                                        println!("Ship {} survey success", ship.ship.symbol);
                                        // STORE RESULTS
                                        let surveys = survey_response.data.surveys;
                                        locked_cl
                                            .ledger
                                            .survey_data
                                            .get_mut(mining_waypoint)
                                            .unwrap()
                                            .insert(surveys);
                                        // SORT RESULTS
                                        println!("Ship {} sorting survey", ship.ship.symbol);
                                        // TODO: GET MARKETS FROM ASTRONOMICON
                                        // locked_cl.ledger.survey_data.get(mining_waypoint).unwrap().sort_surveys_by_market_data(markets);
                                        locked_cl
                                            .ledger
                                            .survey_data
                                            .get_mut(mining_waypoint)
                                            .unwrap()
                                            .sort_surveys_for_items(
                                                &active_contract_delivery_items,
                                            );
                                        // MINE WITH SURVEY
                                        println!("Ship {} mining with survey", ship.ship.symbol);
                                        let target_survey = locked_cl.ledger.survey_data
                                            [mining_waypoint]
                                            .most_recent_surveys[0]
                                            .clone();
                                        extract_resources(
                                            &client,
                                            ship.ship.symbol.as_str(),
                                            Some(ExtractResourcesRequest {
                                                survey: Some(Box::new(target_survey)),
                                            }),
                                        )
                                        .await
                                        .unwrap();
                                        continue;
                                    }
                                    Err(e) => {
                                        // IF SURVEY FAILURE
                                        println!("Ship {} survey on cooldown", ship.ship.symbol);
                                        // STORE COOLDOWN
                                        // TODO: Can I get the data from the error without it being generated from the definition? Probably need to ask them for the ability to catalogue the error responses...
                                        continue;
                                    }
                                }
                            }
                        }
                    } else {
                        // IF NEVER SURVEYED AT WAYPOINT
                        // Survey
                        println!(
                            "Ship {} never surveyed at waypoint, try survey",
                            ship.ship.symbol
                        );
                        let survey_result = create_survey(&client, ship.ship.symbol.as_str()).await;
                        match survey_result {
                            Ok(survey_response) => {
                                // IF SURVEY SUCCESS
                                println!("Ship {} survey success", ship.ship.symbol);
                                // STORE RESULTS
                                let surveys = survey_response.data.surveys;
                                locked_cl
                                    .ledger
                                    .survey_data
                                    .get_mut(mining_waypoint)
                                    .unwrap()
                                    .insert(surveys);
                                // SORT RESULTS
                                println!("Ship {} sorting survey", ship.ship.symbol);
                                // TODO: GET MARKETS FROM ASTRONOMICON
                                // locked_cl.ledger.survey_data.get(mining_waypoint).unwrap().sort_surveys_by_market_data(markets);
                                locked_cl
                                    .ledger
                                    .survey_data
                                    .get_mut(mining_waypoint)
                                    .unwrap()
                                    .sort_surveys_for_items(&active_contract_delivery_items);
                                // MINE WITH SURVEY
                                println!("Ship {} mining with survey", ship.ship.symbol);
                                let target_survey = locked_cl.ledger.survey_data[mining_waypoint]
                                    .most_recent_surveys[0]
                                    .clone();
                                extract_resources(
                                    &client,
                                    ship.ship.symbol.as_str(),
                                    Some(ExtractResourcesRequest {
                                        survey: Some(Box::new(target_survey)),
                                    }),
                                )
                                .await
                                .unwrap();
                                continue;
                            }
                            Err(e) => {
                                // IF SURVEY FAILURE
                                println!(
                                    "Ship {} survey on cooldown or hit error: {}",
                                    ship.ship.symbol, e
                                );
                                // STORE COOLDOWN
                                // TODO: Can I get the data from the error without it being generated from the definition? Probably need to ask them for the ability to catalogue the error responses...
                                continue;
                            }
                        }
                    }
                } else if ship.ship.nav.waypoint_symbol == deliver_waypoint {
                    // IF AT DELIVERY SITE
                    println!("Ship {} at delivery site", ship.ship.symbol);
                    if ship.ship.cargo.units == 0 {
                        // IF CARGO EMPTY
                        println!("Ship {} cargo empty", ship.ship.symbol);
                        // BUY FUEL
                        println!("Ship {} refuel", ship.ship.symbol);
                        refuel_ship(&client, ship.ship.symbol.as_str())
                            .await
                            .unwrap();

                        // GO MINE
                        println!("Ship {} navigate to mine", ship.ship.symbol);
                        navigate_ship(
                            &client,
                            ship.ship.symbol.as_str(),
                            Some(NavigateShipRequest {
                                waypoint_symbol: mining_waypoint.to_string(),
                            }),
                        )
                        .await
                        .unwrap();
                    } else {
                        // IF CARGO NOT EMPTY
                        println!(
                            "Ship {} cargo not empty, attempt to deliver",
                            ship.ship.symbol
                        );
                        // SELL/DELIVER
                        // TODO: SELL LOGIC
                        let contract_id = "clbi634e50007s60jlv42a0q6";
                        deliver_contract(
                            &client,
                            contract_id,
                            Some(DeliverContractRequest {
                                ship_symbol: ship.ship.symbol,
                                trade_symbol: ship.ship.cargo.inventory[0].symbol.to_string(),
                                units: ship.ship.cargo.inventory[0].units,
                            }),
                        )
                        .await
                        .unwrap();
                    }
                } else {
                    println!(
                        "Ship {} at neither mining_waypoint {}, nor delivery waypoint{}, at: {}",
                        ship.ship.symbol,
                        mining_waypoint,
                        deliver_waypoint,
                        ship.ship.nav.waypoint_symbol
                    );
                    println!("Ship {} navigate to mine", ship.ship.symbol);
                    navigate_ship(
                        &client,
                        ship.ship.symbol.as_str(),
                        Some(NavigateShipRequest {
                            waypoint_symbol: mining_waypoint.to_string(),
                        }),
                    )
                    .await
                    .unwrap();
                }
                // println!("tx travel to X1-OE-A005");
                // drop(locked_cl);
                // match duty_tx.send(Duty::new(
                //     DutyClass::Fly,
                //     "X1-OE-A005".to_string(),
                //     _client.clone(),
                //     cl.clone(),
                // )) {
                //     Ok(_) => println!("tx travel to X1-OE-A005 SENT"),
                //     Err(reserr) => println!("tx ERR {:#?}", reserr),
                // }
            }

            // TEMP: If explorer, conduct exploration
            if ship.role == ShipRole::Explorer {
                // TODO
            }
        }
    }
    Ok(())
}
