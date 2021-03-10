mod config;

use actix_web::{middleware, web, App, HttpServer};
use dotenv::dotenv;
use slog::info;
use tokio_postgres::NoTls;

use crate::config::{Config, AppState};

async fn index(state: web::Data<AppState>) -> &'static str {
    info!(state.log, "GET `/` page");

    "Hello from Rust!"
}

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
            .route("/", web::get().to(index))
    })
    .bind(format!("{}:{}", config.server.host, config.server.port))?
    .run()
    .await
}
