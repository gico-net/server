use crate::db::get_client;
use crate::errors::{AppError, AppErrorType};
use chrono::NaiveDateTime;
use deadpool_postgres::{Client, Pool};
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use uuid::Uuid;

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

}
