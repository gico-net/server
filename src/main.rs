mod config;
mod db;
mod errors;
mod helpers;

mod email;
mod repository;

use actix_web::{middleware, App, HttpServer};
use dotenv::dotenv;
use slog::info;
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
            .configure(repository::routes::config)
            .configure(email::routes::config)
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}
