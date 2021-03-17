use crate::commit::models::Commit;
use crate::config::AppState;
use crate::errors::AppErrorResponse;

use actix_web::{web, HttpResponse, Responder};
use slog::info;

/// Endpoint used for getting all commits
async fn index(state: web::Data<AppState>) -> impl Responder {
    info!(state.log, "GET /commit/");
    let result = Commit::find_all(state.pool.clone()).await;

    match result {
        Ok(commits) => HttpResponse::Ok().json(commits),
        _ => HttpResponse::BadRequest().json(AppErrorResponse {
            detail: "Error trying to read all commits from database"
                .to_string(),
        }),
    }
}

/// Routes for commits
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/commit")
            .service(web::resource("{_:/?}").route(web::get().to(index))),
    );
}
