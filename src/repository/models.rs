use crate::db::get_client;
use crate::errors::{AppError, AppErrorType};
use chrono::NaiveDateTime;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use uuid::Uuid;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "repository")]
pub struct Repository {
    pub id: Uuid,
    pub url: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub uploader_ip: String,
}

impl Repository {
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
}