use crate::config::AppState;
use crate::email::models::Email;
use crate::errors::AppErrorResponse;
use actix_web::{web, HttpResponse, Responder};
use slog::info;

/// Endpoint used for retrieve all emails
async fn index(state: web::Data<AppState>) -> impl Responder {
    let result = Email::find_all(state.pool.clone()).await;
    info!(state.log, "GET /email/");

    match result {
        Ok(emails) => HttpResponse::Ok().json(emails),
        _ => HttpResponse::BadRequest().json(AppErrorResponse {
            detail: "Error trying to read all emails from database"
                .to_string(),
        }),
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/email")
            .service(web::resource("{_:/?}").route(web::get().to(index))),
    );
}
