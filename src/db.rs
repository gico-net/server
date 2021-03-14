use crate::errors::AppError;
use deadpool_postgres::{Client, Pool, PoolError};

/// Return a valid `Client` to make SQL queries
pub async fn get_client(pool: Pool) -> Result<Client, AppError> {
    pool.get()
        .await
        .map_err(|err: PoolError| AppError::from(err))
}
