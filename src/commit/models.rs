use crate::db::get_client;
use crate::errors::{AppError, AppErrorType};

use chrono::{DateTime, Local};
use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "commit")]
/// Commit model
pub struct Commit {
    pub hash: String,
    pub tree: Option<String>,
    pub text: String,
    pub date: DateTime<Local>,
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

    // Find a commit that it has an hash equals to `hash`
    pub async fn find(pool: Pool, hash: String) -> Result<Commit, AppError> {
        let client = get_client(pool.clone()).await.unwrap();
        let statement = client
            .prepare("SELECT * FROM commit WHERE hash = $1")
            .await?;

        let commit = client
            .query_opt(&statement, &[&hash])
            .await?
            .map(|row| Commit::from_row_ref(&row).unwrap());

        match commit {
            Some(commit) => Ok(commit),
            None => Err(AppError {
                error_type: AppErrorType::NotFoundError,
                cause: None,
                message: Some("Commit not found".to_string()),
            }),
        }
    }

    /// Find a commit and delete it, but before check if "Authorization"
    /// matches with SECRET_KEY
    pub async fn delete(
        pool: Pool,
        hash: &String,
    ) -> Result<Commit, AppError> {
        let client = get_client(pool.clone()).await.unwrap();
        let statement = client
            .prepare(
                "
                DELETE FROM commit
                WHERE hash=$1
                RETURNING *
                ",
            )
            .await?;

        let commit = client
            .query_opt(&statement, &[&hash])
            .await?
            .map(|row| Commit::from_row_ref(&row).unwrap());

        match commit {
            Some(commit) => Ok(commit),
            None => Err(AppError {
                error_type: AppErrorType::NotFoundError,
                cause: None,
                message: Some("Commit not found".to_string()),
            }),
        }
    }

    /// Create commits from an array
    pub async fn create(
        pool: Pool,
        commits: Vec<Commit>,
    ) -> Result<Vec<Commit>, AppError> {
        let client = get_client(pool.clone()).await.unwrap();
        let mut raw_query = "INSERT INTO commit VALUES".to_string();

        for commit in commits {
            let tree = match commit.tree {
                Some(t) => format!("'{}'", t),
                None => "NULL".to_string(),
            };
            raw_query += &format!(
                "('{}', {}, E'{}', '{}', '{}', E'{}', '{}', E'{}', '{}'),",
                commit.hash,
                tree,
                commit.text.replace("'", "\\'"),
                commit.date,
                commit.author_email,
                commit.author_name.replace("'", "\\'"),
                commit.committer_email,
                commit.committer_name.replace("'", "\\'"),
                commit.repository_url
            )[..]
        }

        // Remove the last `,`
        let _ = raw_query.pop();

        // TODO: write query with &commits and parameter. Need to implement
        // ToSql trait for `Commit` model
        // let statement = client.prepare(&query[..]).await?;
        // client.query(&statement, &[&commits]

        let statement = client.prepare(&raw_query[..]).await?;
        let result = client
            .query(&statement, &[])
            .await?
            .iter()
            .map(|row| Commit::from_row_ref(row).unwrap())
            .collect::<Vec<Commit>>();

        Ok(result)
    }
}
