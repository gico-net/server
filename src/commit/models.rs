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

/// Model used for 'most authors' function
#[derive(Serialize, Deserialize)]
pub struct CommitNumAuthor {
    pub num: i64,
    pub author_email: String,
    pub author_name: String,
}

impl Commit {
    /// Find all commits. Order them by descrescent `date` field
    /// `commit_hash` is used to search commit that matches with some sha codes
    pub async fn find_all(
        pool: Pool,
        commit_hash: &String,
    ) -> Result<Vec<Commit>, AppError> {
        let client = get_client(pool.clone()).await.unwrap();

        let query;
        let hash;
        if commit_hash != "" {
            hash = format!("%{}%", commit_hash);
            query = "SELECT * FROM commit WHERE hash LIKE $1 ORDER BY date DESC LIMIT 300";
        } else {
            hash = String::new();
            query = "SELECT * FROM commit ORDER BY date DESC LIMIT 300"
        }

        let statement = client.prepare(query).await?;

        let commits;
        if hash != "" {
            commits = client.query(&statement, &[&hash]).await?;
        } else {
            commits = client.query(&statement, &[]).await?;
        }

        let result = commits
            .iter()
            .map(|row| Commit::from_row_ref(row).unwrap())
            .collect::<Vec<Commit>>();

        Ok(result)
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
                commit.text.replace("\\'", "'").replace("'", "\\'"),
                commit.date,
                commit.author_email,
                commit.author_name.replace("\\'", "'").replace("'", "\\'"),
                commit.committer_email,
                commit
                    .committer_name
                    .replace("\\'", "'")
                    .replace("'", "\\'"),
                commit.repository_url
            )[..]
        }

        // Remove the last `,`
        let _ = raw_query.pop();
        raw_query += " RETURNING *";

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

    /// Returns a ranking of authors of his commits number
    pub async fn most_authors(
        pool: Pool,
    ) -> Result<Vec<CommitNumAuthor>, AppError> {
        let client = get_client(pool.clone()).await.unwrap();
        let statement = client.prepare(
                "SELECT COUNT(hash) as num, author_email, author_name FROM commit
                GROUP BY author_email, author_name ORDER BY COUNT(hash) DESC"
            ).await?;

        let authors = client
            .query(&statement, &[])
            .await?
            .iter()
            .map(|row| CommitNumAuthor {
                num: row.get(0),
                author_email: row.get(1),
                author_name: row.get(2),
            })
            .collect::<Vec<CommitNumAuthor>>();

        Ok(authors)
    }
}
