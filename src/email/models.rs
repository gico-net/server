use crate::db::get_client;
use crate::errors::{AppError, AppErrorType};

use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

use hex;
use md5::{Digest, Md5};

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "email")]
/// Emails model
pub struct Email {
    pub email: String,
    pub hash_md5: String,
}

// Struct used to creare a new email
#[derive(Serialize, Deserialize)]
pub struct EmailData {
    pub email: String,
}

impl Email {
    /// Find all emails, returns email and its MD5 hash
    pub async fn find_all(pool: Pool) -> Result<Vec<Email>, AppError> {
        let client = get_client(pool.clone()).await.unwrap();
        let statement = client.prepare("SELECT * FROM email").await?;

        let emails = client
            .query(&statement, &[])
            .await?
            .iter()
            .map(|row| Email::from_row_ref(row).unwrap())
            .collect::<Vec<Email>>();

        Ok(emails)
    }

    /// Search an email
    pub async fn search(
        pool: Pool,
        email: &String,
    ) -> Result<Email, AppError> {
        let client = get_client(pool.clone()).await.unwrap();

        let statement =
            client.prepare("SELECT * FROM email WHERE email=$1").await?;

        let email = client
            .query_opt(&statement, &[&email])
            .await?
            .map(|row| Email::from_row_ref(&row).unwrap());

        match email {
            Some(email) => Ok(email),
            None => Err(AppError {
                error_type: AppErrorType::NotFoundError,
                cause: None,
                message: Some("Email not found".to_string()),
            }),
        }
    }

    /// Create new email
    pub async fn create(
        pool: Pool,
        data: &EmailData,
    ) -> Result<Email, AppError> {
        // Search an email  that matches with that string, because if it's
        // exists, the server cannot create a clone
        match Email::search(pool.clone(), &data.email).await {
            Ok(_) => {
                return Err(AppError {
                    message: Some("Email already exists".to_string()),
                    cause: Some("".to_string()),
                    error_type: AppErrorType::AuthorizationError,
                });
            }
            Err(_) => {}
        };

        let client = get_client(pool.clone()).await.unwrap();

        let mut hasher = Md5::new();
        hasher.update(&data.email.as_bytes());
        let hash_final = hasher.finalize();

        let digest = hex::encode(&hash_final.as_slice());

        let statement = client
            .prepare("INSERT INTO email VALUES ($1, $2) RETURNING *")
            .await?;

        let email = client
            .query_opt(&statement, &[&data.email, &digest])
            .await?
            .map(|row| Email::from_row_ref(&row).unwrap());

        match email {
            Some(email) => Ok(email),
            None => Err(AppError {
                message: Some("Error creating a new email".to_string()),
                cause: Some("Unknown error".to_string()),
                error_type: AppErrorType::DbError,
            }),
        }
    }
}
