//! Postgres bootstrap for the ai-service: connect the pool with the configured
//! limits, then bring the schema up to date with the embedded migrations.

use mca_core::StorageError;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};

use crate::config::DbConfig;
use crate::store::pg::PostgresStore;
use crate::store::pg_apps::PostgresApplications;

/// Opens the pool and runs migrations. Both halves of the store share one pool
/// (the connection limit is per endpoint, not per aggregate).
pub async fn connect(
    cfg: &DbConfig,
) -> Result<(PostgresStore, PostgresApplications), StorageError> {
    let options: PgConnectOptions = cfg
        .url
        .parse()
        .map_err(|err| StorageError::Query(format!("invalid DATABASE_URL: {err}")))?;
    let pool = PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .acquire_timeout(cfg.acquire_timeout)
        .connect_with(options)
        .await
        .map_err(|err| StorageError::PoolExhausted(err.to_string()))?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|err| StorageError::Migration(err.to_string()))?;
    tracing::info!("postgres connection established, migrations applied");
    Ok((
        PostgresStore::new(pool.clone()),
        PostgresApplications::new(pool),
    ))
}
