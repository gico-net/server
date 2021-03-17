use crate::config::AppState;
use crate::errors::{AppError, AppErrorResponse, AppErrorType};
use crate::helpers::uuid_from_string;
use crate::repository::models::{Repository, RepositoryData};
use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use slog::info;
use std::env;
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
    id: web::Path<(String,)>,
) -> impl Responder {
    // I have to match the &id.0 because if it's not a valid Uuid, the server
    // must response "Repository not found".
    // If I pass a not valid Uuid to Repository::find() it raises an error.
    let uuid: Uuid = uuid_from_string(&id.0);

    let result = Repository::find(state.pool.clone(), &uuid).await;
    info!(state.log, "GET /repo/{}/", id.0);

    // `map_err` is also used when repo is not found
    result
        .map(|repo| HttpResponse::Ok().json(repo))
        .map_err(|e| e)
}

/// Endpoint used for delete repository.
/// It uses a SECRET_KEY used like an API key
async fn delete_repo(
    req: HttpRequest,
    state: web::Data<AppState>,
    id: web::Path<(String,)>,
) -> impl Responder {
    let uuid: Uuid = uuid_from_string(&id.0);
    match req.headers().get(header::AUTHORIZATION) {
        Some(x)
            if x.to_str().unwrap()
                != env::var("SECRET_KEY").unwrap_or("".to_string()) =>
        {
            info!(state.log, "DELETE /repo/{}/ 401", id.0);
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
            info!(state.log, "DELETE /repo/{}/ 400", id.0);
            return Ok(HttpResponse::BadRequest().body(""));
        }
    };

    let result = Repository::delete(state.pool.clone(), &uuid).await;
    info!(state.log, "DELETE /repo/{}/", id.0);

    result
        .map(|_| HttpResponse::NoContent().body(""))
        .map_err(|e| e)
}

/// Endpoint used for create new repository
async fn create_repo(
    req: HttpRequest,
    payload: web::Json<RepositoryData>,
    state: web::Data<AppState>,
) -> impl Responder {
    info!(state.log, "POST /repo/");
    let request_from_ip = HttpRequest::peer_addr(&req);
    let result =
        Repository::create(state.pool.clone(), &payload, request_from_ip)
            .await;

    result
        .map(|repo| HttpResponse::Created().json(repo))
        .map_err(|e| e)
}

/// Routes for repository. TODO: create endpoint for UPDATE method
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/repo")
            .service(
                web::resource("{_:/?}")
                    .route(web::get().to(index))
                    .route(web::post().to(create_repo)),
            )
            .service(
                web::resource("/{id}{_:/?}")
                    .route(web::get().to(get_repo))
                    .route(web::delete().to(delete_repo)),
            ),
    );
}
