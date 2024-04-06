mod app;
mod entities;
mod util;

use std::env;

use actix_cors::Cors;
use actix_web::{
    http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    web::Data,
    App, HttpServer,
};
use app::discord::routes::discord_oauth;
use dotenvy::dotenv;
use lazy_static::lazy_static;
use log::{error, info, warn};
use reqwest::Client;
use sea_orm::{Database, DatabaseConnection};
use url::Url;

use crate::app::{
    anilist::routes::anilist_redirect,
    discord::routes::{discord_logout, discord_verify},
};

lazy_static! {
    static ref HTTP_CLIENT: Client = Client::new();

    // client environment variables
    static ref MAKISHIMA_ID: String = env::var("MAKISHIMA_ID").expect("MAKISHIMA_ID is not set");
    static ref MAKISHIMA_SECRET: String = env::var("MAKISHIMA_SECRET").expect("MAKISHIMA_SECRET is not set");
    static ref MAKISHIMA_REDIRECT: String = env::var("MAKISHIMA_REDIRECT").expect("MAKISHIMA_REDIRECT is not set");
    static ref MAKISHIMA_SIGKEY: String = env::var("MAKISHIMA_SIGKEY").expect("MAKISHIMA_SIGKEY is not set");

    // anilist oauth
    static ref ANILIST_ID: String = env::var("ANILIST_ID").expect("ANILIST_ID is not set");
    static ref ANILIST_SECRET: String = env::var("ANILIST_SECRET").expect("ANILIST_SECRET is not set");
    static ref ANILIST_REDIRECT: String = env::var("ANILIST_REDIRECT").expect("ANILIST_REDIRECT is not set");

    // database
    static ref DB_URI: String = env::var("DB_URI").expect("DB_URI is not set");

    // endpoints
    static ref DISCORD_ENDPOINT: Url = Url::parse("https://discord.com/").unwrap();
    static ref ANILIST_ENDPOINT: Url = Url::parse("https://anilist.co/").unwrap();
    static ref ANILIST_GRAPHQL_ENDPOINT: Url = Url::parse("https://graphql.anilist.co/").unwrap();
}

#[derive(Clone)]
struct AppState {
    db: DatabaseConnection,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if dotenv().is_err() {
        warn!("Unable to find a .env file. Using system environment variables...");
    }

    env_logger::init();

    let db = Database::connect(DB_URI.to_string()).await;

    if let Err(err) = db {
        error!("Unable to connect to database: {}", err);
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotConnected,
            "unable to connect to database",
        ));
    }

    info!("Successfully connected to database");
    let state = AppState { db: db.unwrap() };

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:5173")
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![ACCEPT, AUTHORIZATION, CONTENT_TYPE])
                    .supports_credentials(),
            )
            // discord services
            .service(discord_verify)
            .service(discord_logout)
            .service(discord_oauth)
            // anilist services
            .service(anilist_redirect)
            // shared application state
            .app_data(Data::new(state.clone()))
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
