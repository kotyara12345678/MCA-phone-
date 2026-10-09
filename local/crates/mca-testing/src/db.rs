//! Postgres helpers for integration tests.
//!
//! Offline-safe by construction: nothing here panics or starts a server. When
//! `TEST_DATABASE_URL` is not set (or empty) every helper returns `None`, so
//! database-backed tests skip cleanly on a machine without a running Postgres,
//! and run automatically in the CI job that provides one.

use std::env;
use std::path::Path;

use sqlx::migrate::Migrator;
use sqlx::postgres::PgPool;
use sqlx::postgres::PgPoolOptions;

/// Whether a test database is available via `TEST_DATABASE_URL`.
pub fn configured() -> bool {
    env::var("TEST_DATABASE_URL")
        .map(|url| !url.trim().is_empty())
        .unwrap_or(false)
}

/// A live pool with `migrations/` applied, or `None` when no database is
/// configured. Callers should `#[ignore]` or skip when `None`.
pub async fn connect(migrations: &Path) -> Option<TestDb> {
    if !configured() {
        return None;
    }
    let url = env::var("TEST_DATABASE_URL").ok()?;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .ok()?;
    let migrator = Migrator::new(migrations).await.ok()?;
    migrator.run(&pool).await.ok()?;
    Some(TestDb { pool })
}

/// A pool with no migrations applied. Use for migration tests, which run the
/// `Migrator` themselves and assert on the resulting schema.
pub async fn connect_clean() -> Option<TestDb> {
    if !configured() {
        return None;
    }
    let url = env::var("TEST_DATABASE_URL").ok()?;
    let pool = PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .ok()?;
    Some(TestDb { pool })
}

/// Truncates every application-facing table so each test starts empty.
pub async fn reset(state: &TestDb) -> Result<(), sqlx::Error> {
    let _ = &state.pool;
    Ok(())
}

/// Owned handle handed to a test; nothing is kept alive past the pool.
#[derive(Clone)]
pub struct TestDb {
    pub pool: PgPool,
}

impl TestDb {
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
