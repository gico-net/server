use crate::commit::models::Commit;
use crate::config::AppState;
use crate::errors::{AppError, AppErrorResponse, AppErrorType};
use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use slog::info;
use std::env;

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
    hash: web::Path<String>,
) -> impl Responder {
    info!(state.log, "GET /commit/{}/", &hash);

    let result = Commit::find(state.pool.clone(), hash.clone()).await;

    result
        .map(|commit| HttpResponse::Ok().json(commit))
        .map_err(|e| e)
}

/// Endpoint used for delete commitsitory.
/// It uses a SECRET_KEY used like an API key
async fn delete_commit(
    req: HttpRequest,
    state: web::Data<AppState>,
    hash: web::Path<String>,
) -> impl Responder {
    match req.headers().get(header::AUTHORIZATION) {
        Some(x)
            if x.to_str().unwrap()
                != env::var("SECRET_KEY").unwrap_or("".to_string()) =>
        {
            info!(state.log, "DELETE /commit/{}/ 401", &hash);
            return Err(AppError {
                error_type: AppErrorType::AuthorizationError,
                message: Some(
                    "You must provide a valid Authorization".to_string(),
                ),
                cause: None,
            });
        }
        Some(_) => {}
        None => {
            info!(state.log, "DELETE /commit/{}/ 400", &hash);
            return Ok(HttpResponse::BadRequest().body(""));
        }
    };

    let result = Commit::delete(state.pool.clone(), &hash).await;
    info!(state.log, "DELETE /commit/{}/", &hash);

    result
        .map(|_| HttpResponse::NoContent().body(""))
        .map_err(|e| e)
}

/// Routes for commits
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/commit")
            .service(web::resource("/").route(web::get().to(index)))
            .service(
                web::resource("/{hash}/")
                    .route(web::get().to(get_commit))
                    .route(web::delete().to(delete_commit)),
            ),
    );
}
