use std::collections::HashMap;

use crate::config::AppState;
use crate::email::models::{Email, EmailData};
use crate::errors::AppErrorResponse;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
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

// Endpoint used for create new email
async fn create_email(
    payload: web::Json<EmailData>,
    state: web::Data<AppState>,
) -> impl Responder {
    info!(state.log, "POST /email/");
    let result = Email::create(state.pool.clone(), &payload).await;

    result
        .map(|email| HttpResponse::Created().json(email))
        .map_err(|e| e)
}

// Endpoint used for email search
async fn search_email(
    req: HttpRequest,
    state: web::Data<AppState>,
) -> impl Responder {
    let query =
        web::Query::<HashMap<String, String>>::from_query(req.query_string())
            .unwrap();
    let email = match query.get("q") {
        Some(x) => x,
        None => {
            return HttpResponse::NotFound().json(AppErrorResponse {
                detail: "No email found".to_string(),
            });
        }
    };
    let result = Email::search(state.pool.clone(), email).await;
    info!(state.log, "GET /email/search?q={}", email);

    match result {
        Ok(email) => HttpResponse::Ok().json(email),
        _ => HttpResponse::NotFound().json(AppErrorResponse {
            detail: "No email found".to_string(),
        }),
    }
}
pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/email")
            .service(
                web::resource("{_:/?}")
                    .route(web::get().to(index))
                    .route(web::post().to(create_email)),
            )
            .service(
                web::resource("/search{_:/?}")
                    .route(web::get().to(search_email)),
            ),
    );
}
