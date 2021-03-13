use crate::errors::AppError;
use deadpool_postgres::{Client, Pool, PoolError};

pub async fn get_client(pool: Pool) -> Result<Client, AppError> {
    pool.get()
        .await
        .map_err(|err: PoolError| AppError::from(err))
}
