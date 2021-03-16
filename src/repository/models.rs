use crate::db::get_client;
use crate::errors::{AppError, AppErrorType};
use crate::helpers::name_of_git_repository;

use chrono::NaiveDateTime;
use deadpool_postgres::{Client, Pool};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use uuid::Uuid;

use std::net::SocketAddr;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "repository")]
/// Repository model
pub struct Repository {
    pub id: Uuid,
    pub url: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub uploader_ip: String,
}

/// Struct used to create a new repository
#[derive(Serialize, Deserialize)]
pub struct RepositoryData {
    pub url: String,
}

impl Repository {
    /// Find all repositories inside the database.
    /// Make a select query and order the repositories by descrescent updated
    /// datetime
    pub async fn find_all(pool: Pool) -> Result<Vec<Repository>, AppError> {
        let client = get_client(pool.clone()).await.unwrap();
        let statement = client
            .prepare("SELECT * FROM repository ORDER BY updated_at DESC")
            .await?;

        let repos = client
            .query(&statement, &[])
            .await?
            .iter()
            .map(|row| Repository::from_row_ref(row).unwrap())
            .collect::<Vec<Repository>>();

        Ok(repos)
    }

    /// Find a repository with an `id` equals to an Uuid element
    pub async fn find(pool: Pool, id: &Uuid) -> Result<Repository, AppError> {
        let client = get_client(pool.clone()).await.unwrap();
        let statement = client
            .prepare("SELECT * FROM repository WHERE id = $1")
            .await?;

        let repo = client
            .query_opt(&statement, &[&id])
            .await?
            .map(|row| Repository::from_row_ref(&row).unwrap());

        match repo {
            Some(repo) => Ok(repo),
            None => Err(AppError {
                error_type: AppErrorType::NotFoundError,
                cause: None,
                message: Some("Repository not found".to_string()),
            }),
        }
    }

    /// Find a repository and delete it, but before check if "Authorization"
    /// matches with SECRET_KEY
    pub async fn delete(
        pool: Pool,
        id: &Uuid,
    ) -> Result<Repository, AppError> {
        let client = get_client(pool.clone()).await.unwrap();
        let statement = client
            .prepare(
                "
                DELETE FROM repository
                WHERE id=$1
                RETURNING *
                ",
            )
            .await?;

        let repo = client
            .query_opt(&statement, &[&id])
            .await?
            .map(|row| Repository::from_row_ref(&row).unwrap());

        match repo {
            Some(repo) => Ok(repo),
            None => Err(AppError {
                error_type: AppErrorType::NotFoundError,
                cause: None,
                message: Some("Repository not found".to_string()),
            }),
        }
    }

    /// Search a repository by its url
    async fn search(
        client: &Client,
        url: String,
    ) -> Result<Repository, AppError> {
        let statement = client
            .prepare("SELECT * FROM repository WHERE url=$1")
            .await?;

        let repo = client
            .query_opt(&statement, &[&url])
            .await?
            .map(|row| Repository::from_row_ref(&row).unwrap());

        match repo {
            Some(repo) => Ok(repo),
            None => Err(AppError {
                error_type: AppErrorType::NotFoundError,
                cause: None,
                message: Some("Repository not found".to_string()),
            }),
        }
    }

    /// Create a new repository. It uses RepositoryData as support struct
    pub async fn create(
        pool: Pool,
        data: &RepositoryData,
        uploader_ip: Option<SocketAddr>,
    ) -> Result<Repository, AppError> {
        let client = get_client(pool.clone()).await.unwrap();

        let repo_name: String = match name_of_git_repository(&data.url) {
            Some(path) => path,
            None => {
                return Err(AppError {
                    message: Some("Repository not found".to_string()),
                    cause: Some("".to_string()),
                    error_type: AppErrorType::NotFoundError,
                });
            }
        };

        // Search a repository that matches with that url, because if it's
        // exists, the server do not create a clone
        let repo_search = Repository::search(&client, repo_name.clone()).await;
        match repo_search {
            Ok(_) => {
                return Err(AppError {
                    message: Some("Repository already exists".to_string()),
                    cause: Some("".to_string()),
                    error_type: AppErrorType::AuthorizationError,
                });
            }
            Err(_) => {}
        };

        let statement = client
            .prepare("
                INSERT INTO repository(id, url, uploader_ip) VALUES($1, $2, $3) RETURNING *
            ").await?;

        // Create a new UUID v4
        let uuid = Uuid::new_v4();

        // Match the uploader ip
        let user_ip = match uploader_ip {
            Some(ip) => ip.to_string(),
            None => {
                return Err(AppError {
                    message: Some("Failed to fetch uploader ip".to_string()),
                    cause: Some("".to_string()),
                    error_type: AppErrorType::AuthorizationError,
                })
            }
        };

        let repo = client
            .query(&statement, &[&uuid, &repo_name, &user_ip])
            .await?
            .iter()
            .map(|row| Repository::from_row_ref(row).unwrap())
            .collect::<Vec<Repository>>()
            .pop();

        match repo {
            Some(repo) => Ok(repo),
            None => Err(AppError {
                message: Some("Error creating a new repository".to_string()),
                cause: Some("Unknown error".to_string()),
                error_type: AppErrorType::DbError,
            }),
        }
    }
}
