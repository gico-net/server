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

// Endpoint used for getting one commit
async fn get_commit(
    state: web::Data<AppState>,
    hash: web::Path<(String,)>,
) -> impl Responder {
    info!(state.log, "GET /commit/{}/", &hash.0);

    let result = Commit::find(state.pool.clone(), hash.0.clone()).await;

    result
        .map(|commit| HttpResponse::Ok().json(commit))
        .map_err(|e| e)
}

/// Routes for commits
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/commit")
            .service(web::resource("{_:/?}").route(web::get().to(index)))
            .service(
                web::resource("/{hash}{_:/?}")
                    .route(web::get().to(get_commit)),
            ),
    );
}
