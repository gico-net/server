use crate::config::AppState;
use crate::errors::AppErrorResponse;
use crate::repository::models::Repository;
use actix_web::{web, HttpResponse, Responder};
use slog::info;
use uuid::Uuid;

/// Endpoint used for retrieve all repositories
async fn index(state: web::Data<AppState>) -> impl Responder {
    let result = Repository::find_all(state.pool.clone()).await;
    info!(state.log, "GET /repo/");

    // If raises an `Err`, returns an error in JSON format
    match result {
        Ok(repos) => HttpResponse::Ok().json(repos),
        _ => HttpResponse::BadRequest().json(AppErrorResponse {
            detail: "Error trying to read all repositories from database"
                .to_string(),
        }),
    }
}

/// Endpoint used for retrieve a repository that matches with an `id`.
/// It is a String, casted in an Uuid format.
async fn get_repo(
    state: web::Data<AppState>,
    id: web::Path<(Uuid,)>,
) -> impl Responder {
    let result = Repository::find(state.pool.clone(), &id.0).await;
    info!(state.log, "GET /repo/{}/", id.0);

    // `map_err` is also used when repo is not found
    result
        .map(|repo| HttpResponse::Ok().json(repo))
        .map_err(|e| e)
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/repo{_:/?}").route(web::get().to(index)))
        .service(
            web::resource("/repo/{id}{_:/?}").route(web::get().to(get_repo)),
        );
}
