mod services;
mod utils;

use crate::services::*;
use crate::utils::initialize_database;
use actix_cors::Cors;
use actix_web::middleware::Logger;
use actix_web::{App, HttpServer, Scope};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenvy::dotenv().ok();

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let database = initialize_database().await;

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header()
                    .supports_credentials(),
            )
            .app_data(database.clone())
            .wrap(Logger::default())
            .service(Scope::new("/").service(get::index).service(post::index))
            .service(
                Scope::new("/user")
                    .service(get::user)
                    .service(post::user)
                    .service(put::user)
                    .service(delete::user),
            )
            .service(
                Scope::new("/article")
                    .service(get::article)
                    .service(post::article)
                    .service(put::article)
                    .service(delete::article),
            )
    })
    .bind((
        "0.0.0.0",
        env::var("PORT")
            .expect("Failed to get PORT")
            .parse()
            .expect("Failed to parse PORT"),
    ))?
    .run()
    .await
}
