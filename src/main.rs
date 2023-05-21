use dotenv;
use surrealdb::engine::remote::ws::Ws;
use surrealdb::opt::auth::Root;
use surrealdb::Surreal;

// #[derive(Debug, Serialize)]
// struct Name<'a> {
//     first: &'a str,
//     last: &'a str,
// }

// #[derive(Debug, Serialize)]
// struct Person<'a> {
//     title: &'a str,
//     name: Name<'a>,
//     marketing: bool,
// }

// #[derive(Debug, Serialize)]
// struct Responsibility {
//     marketing: bool,
// }

// #[derive(Debug, Deserialize)]
// struct Record {
//     #[allow(dead_code)]
//     id: Thing,
// }

use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

mod models;
mod types;
mod yaml_util;

#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "starscript=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::debug!("--Intializing Starscript--");
    // Load env files
    tracing::debug!("--Loading Env Vars--");
    dotenv::from_filename("surreal_secrets.env").ok();
    let surreal_user = std::env::var("SURREAL_USER").expect("SURREAL_USER must be set");
    let surreal_pass = std::env::var("SURREAL_PASS").expect("SURREAL_PASS must be set");

    // Connect to the server
    tracing::debug!("--Connecting to DB--");
    let db = Surreal::new::<Ws>("localhost:8000")
        .await
        .expect("Could not connect to DB");

    // Signin as a namespace, database, or root user
    db.signin(Root {
        username: surreal_user.as_str(),
        password: surreal_pass.as_str(),
    })
    .await
    .expect("Could not sign in to DB");

    // Select a specific namespace / database
    db.use_ns("starscript")
        .use_db("starscript")
        .await
        .expect("Could not select starscript namespace or db");

    tracing::debug!("--DB Connected and Ready--");

    tracing::debug!("--Loading Rules--");
    let staleness_rules: types::StalenessRules = yaml_util::load_staleness_rules();
    tracing::debug!("Staleness Rules: {:#?}", staleness_rules);

    tracing::debug!("--==Starscript Ready==--");

    // // Create a new person with a random id
    // let created: Record = db
    //     .create("person")
    //     .content(Person {
    //         title: "Founder & CEO",
    //         name: Name {
    //             first: "Tobie",
    //             last: "Morgan Hitchcock",
    //         },
    //         marketing: true,
    //     })
    //     .await?;
    // dbg!(created);

    // // Update a person record with a specific id
    // let updated: Record = db
    //     .update(("person", "jaime"))
    //     .merge(Responsibility { marketing: true })
    //     .await?;
    // dbg!(updated);

    // // Select all people records
    // let people: Vec<Record> = db.select("person").await?;
    // dbg!(people);

    // // Perform a custom advanced query
    // let groups = db
    //     .query("SELECT marketing, count() FROM type::table($table) GROUP BY marketing")
    //     .bind(("table", "person"))
    //     .await?;
    // dbg!(groups);

    // Ok(())
}
