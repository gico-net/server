mod config;

use actix_web::{web, App, HttpResponse, HttpServer};
use dotenv::dotenv;
use tokio_postgres::NoTls;

use crate::config::Config;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let config = Config::from_env().unwrap();
    let pool = config.pg.create_pool(NoTls).unwrap();

    HttpServer::new(move || {
        App::new().data(pool.clone()).service(
            web::resource("/")
                .to(|| HttpResponse::Ok().body("Hello from Rust!")),
        )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
