use crate::db::get_client;
use crate::errors::AppError;

use deadpool_postgres::Pool;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_pg_mapper_derive::PostgresMapper;

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
}
