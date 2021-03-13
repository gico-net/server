use crate::config::AppState;
use crate::errors::AppErrorResponse;
use crate::repository::models::Repository;
use actix_web::{web, HttpResponse, Responder};
use slog::info;

async fn index(state: web::Data<AppState>) -> impl Responder {
    let result = Repository::find_all(state.pool.clone()).await;
    info!(state.log, "GET /repo/");
    match result {
        Ok(repos) => HttpResponse::Ok().json(repos),
        _ => HttpResponse::BadRequest().json(AppErrorResponse {
            detail: "Error trying to read all repositories from database"
                .to_string(),
        }),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/repo{_:/?}").route(web::get().to(index)))
}
