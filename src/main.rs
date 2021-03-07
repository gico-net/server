use actix_web::{web, App, HttpResponse, HttpServer};
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(
            web::resource("/")
                .to(|| HttpResponse::Ok().body("Hello from Rust!")),
        )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
