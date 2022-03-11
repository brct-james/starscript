use spacedustrs::client::Client;
// use std::collections::HashMap;
// use std::thread;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Setup Game Client
    let http_client = spacedustrs::client::get_http_client(None);

    // // Register agent
    // let claim_agent_response = spacedustrs::client::claim_agent(
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

    println!("New Client");
    let client = Client::new(
        http_client,
        "https://v2-0-0.alpha.spacetraders.io".to_string(),
        "GREEN".to_string(),
        "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJpZGVudGlmaWVyIjoiR1JFRU4iLCJpYXQiOjE2NDcwMTY1NjMsInN1YiI6ImFnZW50LXRva2VuIn0.S-9AfG_asd21tsdGf9TF-cwML32x-TFd-b2n9WT21CKA3gkS9qhR15Zng9I2chv92NRriUGUDVb3flc-nZfnbDrMK_iBUbHT7oLiUu1X4Rr9HumsHUdSEltpVGxTvRm6-0udRgLuy9ndoXCxomUsTruszqdRZ5BJb9-2OYcP_kU6FnYcERDGoKNn6jISmPCaSnSs8nCDw5dbSrDF16mAJiGozJlx9j1gDUHWzeQZF7k4fonPxcLPGQjSa4mKIMaYYCh5oATW3wMh5qnXb-iz-wiwHZ7aXd1jkmDVQzeFXYqLpNf1jjQOXXdqEcZ_lFe79Mgeg1vuNtJDZpPNh-KC7P7YdC_F-7DYA82x6uYDPN8bwxcPd5uNmw0lZr5_C0lUI_z8-igPurxDBLizwjBdMdjIaqY2YSjEV_zocRy-I-N_0c43Dc9a5zZoFFH0DPwFrR2c9pp3tSkFsRMHp86SVlASIDXCQlgLvlNoDORi79dVR9ap64JgK3z-ttoJ_v90".to_string(),
    );

    println!("Match");
    match client.get_my_agent_details().await {
        Ok(res) => {
            println!("ok {:#?}", res);
        }
        Err(res_err) => {
            println!("err {:?}", res_err);
        }
    }

    Ok(())
}
