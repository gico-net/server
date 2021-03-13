use crate::config::AppState;
use crate::repository::models::Repository;
use actix_web::{web, HttpResponse, Responder};
use slog::info;

async fn index(state: web::Data<AppState>) -> impl Responder {
    let result = Repository::find_all(state.pool.clone()).await;
    match result {
        Ok(repos) => {
            info!(state.log, "GET /repo/ 200");
            HttpResponse::Ok().json(repos)
        }
        _ => {
            info!(state.log, "GET /repo/ 500");
            HttpResponse::BadRequest()
                .body("Error trying to read all repositories from database")
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/repo/").route(web::get().to(index)));
}
