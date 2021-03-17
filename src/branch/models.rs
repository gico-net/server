use crate::db::get_client;
use crate::errors::{AppError, AppErrorType};

use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;
use uuid::Uuid;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "branch")]
/// Branch model
pub struct Branch {
    pub id: Uuid,
    pub name: String,
    pub repository_id: Uuid,
    pub head: String,
}

impl Branch {
    /// Find all branches
    pub async fn find_all(pool: Pool) -> Result<Vec<Branch>, AppError> {
        let client = get_client(pool.clone()).await.unwrap();
        let statement = client.prepare("SELECT * FROM branch").await?;

        let branches = client
            .query(&statement, &[])
            .await?
            .iter()
            .map(|row| Branch::from_row_ref(row).unwrap())
            .collect::<Vec<Branch>>();

        Ok(branches)
    }

    /// Find a branch with an `id` equals to an Uuid element
    pub async fn find(pool: Pool, id: &Uuid) -> Result<Branch, AppError> {
        let client = get_client(pool.clone()).await.unwrap();
        let statement =
            client.prepare("SELECT * FROM branch WHERE id = $1").await?;

        let branch = client
            .query_opt(&statement, &[&id])
            .await?
            .map(|row| Branch::from_row_ref(&row).unwrap());

        match branch {
            Some(branch) => Ok(branch),
            None => Err(AppError {
                error_type: AppErrorType::NotFoundError,
                cause: None,
                message: Some("Branch not found".to_string()),
            }),
        }
    }

    /// Find all branches of a repository
    pub async fn find_by_repo(
        pool: Pool,
        repo: &Uuid,
    ) -> Result<Vec<Branch>, AppError> {
        let client = get_client(pool.clone()).await.unwrap();
        let statement = client
            .prepare("SELECT * FROM branch WHERE repository_id=$1")
            .await?;

        let branches = client
            .query(&statement, &[&repo])
            .await?
            .iter()
            .map(|row| Branch::from_row_ref(row).unwrap())
            .collect::<Vec<Branch>>();

        Ok(branches)
    }

    /// Find a branch and delete it, but before check if "Authorization"
    /// matches with SECRET_KEY
    pub async fn delete(pool: Pool, id: &Uuid) -> Result<Branch, AppError> {
        let client = get_client(pool.clone()).await.unwrap();
        let statement = client
            .prepare(
                "
                DELETE FROM branch
                WHERE id=$1
                RETURNING *
                ",
            )
            .await?;

        let branch = client
            .query_opt(&statement, &[&id])
            .await?
            .map(|row| Branch::from_row_ref(&row).unwrap());

        match branch {
            Some(branch) => Ok(branch),
            None => Err(AppError {
                error_type: AppErrorType::NotFoundError,
                cause: None,
                message: Some("Branch not found".to_string()),
            }),
        }
    }
}
