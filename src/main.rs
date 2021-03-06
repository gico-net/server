mod config;
mod db;
mod errors;
mod helpers;

mod git;

mod branch;
mod commit;
mod email;
mod repository;

use actix_cors::Cors;
use actix_web::{http::header, middleware, App, HttpServer};
use dotenv::dotenv;
use slog::info;
use std::env;
use tokio_postgres::NoTls;

use crate::config::{AppState, Config};

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = Config::from_env().unwrap();
    let pool = config.pg.create_pool(NoTls).unwrap();
    let log = Config::logging();

    info!(
        log,
        "Starting server at http://{}:{}",
        config.server.host,
        config.server.port
    );

    HttpServer::new(move || {
        App::new()
            .data(AppState {
                pool: pool.clone(),
                log: log.clone(),
            })
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin(&env::var("CLIENT").unwrap())
                    .allowed_methods(vec!["GET", "POST", "DELETE"])
                    .allowed_headers(vec![
                        header::AUTHORIZATION,
                        header::ACCEPT,
                    ])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .configure(repository::routes::config)
            .configure(email::routes::config)
            .configure(commit::routes::config)
            .configure(branch::routes::config)
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}
