use crate::branch::models::Branch;
use crate::config::AppState;
use crate::errors::{AppError, AppErrorResponse, AppErrorType};
use crate::helpers::uuid_from_string;

use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use slog::info;
use std::env;
use uuid::Uuid;

/// Endpoint used for getting all commits
async fn index(state: web::Data<AppState>) -> impl Responder {
    info!(state.log, "GET /branch/");
    let result = Branch::find_all(state.pool.clone()).await;

    match result {
        Ok(branches) => HttpResponse::Ok().json(branches),
        _ => HttpResponse::BadRequest().json(AppErrorResponse {
            detail: "Error trying to read all branches from database"
                .to_string(),
        }),
    }
}

// Endpoint used for getting branches of a repository
async fn get_repo_branch(
    state: web::Data<AppState>,
    repo: web::Path<String>,
) -> impl Responder {
    let uuid: Uuid = uuid_from_string(&repo);
    info!(state.log, "GET /branch/repo/{}/", &uuid);

    let result = Branch::find_by_repo(state.pool.clone(), &uuid).await;

    result
        .map(|branches| HttpResponse::Ok().json(branches))
        .map_err(|e| e)
}

/// Endpoint used for retrieve a repository that matches with an `id`.
/// It is a String, casted in an Uuid format.
async fn get_branch(
    state: web::Data<AppState>,
    id: web::Path<String>,
) -> impl Responder {
    let uuid: Uuid = uuid_from_string(&id);

    let result = Branch::find(state.pool.clone(), &uuid).await;
    info!(state.log, "GET /branch/{}/", id);

    // `map_err` is also used when repo is not found
    result
        .map(|repo| HttpResponse::Ok().json(repo))
        .map_err(|e| e)
}

/// Endpoint used for delete branch.
/// It uses a SECRET_KEY used like an API key
async fn delete_branch(
    req: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<String>,
) -> impl Responder {
    let uuid: Uuid = uuid_from_string(&id);
    match req.headers().get(header::AUTHORIZATION) {
        Some(x)
            if x.to_str().unwrap()
                != env::var("SECRET_KEY").unwrap_or("".to_string()) =>
        {
            info!(state.log, "DELETE /branch/{}/ 401", id);
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
            info!(state.log, "DELETE /branch/{}/ 400", id);
            return Ok(HttpResponse::BadRequest().body(""));
        }
    };

    let result = Branch::delete(state.pool.clone(), &uuid).await;
    info!(state.log, "DELETE /branch/{}/", id);

    result
        .map(|_| HttpResponse::NoContent().body(""))
        .map_err(|e| e)
}

/// Routes for branches
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/branch")
            .service(web::resource("/").route(web::get().to(index)))
            .service(
                web::resource("/repo/{repo_id}/")
                    .route(web::get().to(get_repo_branch)),
            )
            .service(
                web::resource("/{id}/")
                    .route(web::get().to(get_branch))
                    .route(web::delete().to(delete_branch)),
            ),
    );
}
