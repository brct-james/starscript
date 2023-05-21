use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::{self, layer::SubscriberExt, util::SubscriberInitExt};

// use crate::models::Tabled;

// mod controllers;
// mod models;
// mod response;
// mod utils;
// mod yaml_util;

#[tokio::main]
async fn main() {
    let durl = std::env::var("DATABASE_URL").expect("set DATABASE_URL env variable");
    // initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "starscript=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::debug!("--Intializing Server--");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&durl)
        .await
        .expect("unable to connect to database");

    tracing::debug!("--Intializing Database--");

    let drop_strings: Vec<Vec<String>> = vec![
        // models::syndicate::Syndicate::get_drop_strings(),
        // models::user::User::get_drop_strings(),
        // models::world::World::get_drop_strings(),
        // models::fractal::Fractal::get_drop_strings(),
        // models::shattere::Shattere::get_drop_strings(),
        // models::region::Region::get_drop_strings(),
        // models::location::Location::get_drop_strings(),
        // models::connection::Connection::get_drop_strings(),
        // models::npc::NPC::get_drop_strings(),
        // models::ship::Ship::get_drop_strings(),
        // models::commodity::Commodity::get_drop_strings(),
        // models::factory::Factory::get_drop_strings(),
        // models::inventory::FactoryInventory::get_drop_strings(),
        // models::commodity_order::CommodityOrder::get_drop_strings(),
        // models::recipe::Recipe::get_drop_strings(),
        // models::building::Building::get_drop_strings(),
        // models::crafting_order::CraftingOrder::get_drop_strings(),
    ];
    for dsv in drop_strings {
        for ds in dsv {
            tracing::debug!("{}", ds);
            sqlx::query(&ds).execute(&pool).await.unwrap();
        }
    }

    let create_strings: Vec<Vec<String>> = vec![
        // models::syndicate::Syndicate::get_create_strings(),
        // models::user::User::get_create_strings(),
        // models::world::World::get_create_strings(),
        // models::fractal::Fractal::get_create_strings(),
        // models::shattere::Shattere::get_create_strings(),
        // models::region::Region::get_create_strings(),
        // models::location::Location::get_create_strings(),
        // models::connection::Connection::get_create_strings(),
        // models::npc::NPC::get_create_strings(),
        // models::ship::Ship::get_create_strings(),
        // models::commodity::Commodity::get_create_strings(),
        // models::factory::Factory::get_create_strings(),
        // models::inventory::FactoryInventory::get_create_strings(),
        // models::commodity_order::CommodityOrder::get_create_strings(),
        // models::recipe::Recipe::get_create_strings(),
        // models::building::Building::get_create_strings(),
        // models::crafting_order::CraftingOrder::get_create_strings(),
    ];
    for tsv in create_strings {
        for ts in tsv {
            tracing::debug!("{}", ts);
            sqlx::query(&ts).execute(&pool).await.unwrap();
        }
    }

    tracing::debug!("--Populating Database--");

    // yaml_util::load_yaml_to_db(&pool).await;

    // run our app
    tracing::debug!("--==Starscript Ready==--");
}
