use crate::db::get_client;
use crate::errors::AppError;

use chrono::NaiveDateTime;
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "commit")]
/// Commit model
pub struct Commit {
    pub hash: String,
    pub tree: String,
    pub text: String,
    pub date: NaiveDateTime,
    pub author_email: String, // Reference to Email
    pub author_name: String,
    pub committer_email: String, // Reference to Email
    pub committer_name: String,
    pub repository_url: String, // Reference to Repository
}

impl Commit {
    /// Find all commits. Order them by descrescent `date` field
    pub async fn find_all(pool: Pool) -> Result<Vec<Commit>, AppError> {
        let client = get_client(pool.clone()).await.unwrap();
        let statement = client
            .prepare("SELECT * FROM commit ORDER BY date DESC")
            .await?;

        let commits = client
            .query(&statement, &[])
            .await?
            .iter()
            .map(|row| Commit::from_row_ref(row).unwrap())
            .collect::<Vec<Commit>>();

        Ok(commits)
    }
}
