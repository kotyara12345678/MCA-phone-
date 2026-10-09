//! Postgres session-transcript store; `payload` columns hold the serialized
//! aggregate, scalar columns feed the `where id = $1` lookups. Applications
//! and emails live in `pg_apps.rs`; row helpers in `pg_helpers.rs`.

use async_trait::async_trait;
use mca_core::domain::message::Message;
use mca_core::domain::session::Session;
use mca_core::{SessionId, StorageError};
use sqlx::{postgres::PgRow, PgPool};

use super::pg_helpers::{decode, decode_many, query_error, serde_error};
use super::SessionStore;

pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SessionStore for PostgresStore {
    async fn insert_session(&self, session: &Session) -> Result<(), StorageError> {
        let payload = serde_json::to_value(session).map_err(serde_error)?;
        sqlx::query(
            "insert into sessions (id, call_id, payload, started_at) values ($1, $2, $3, $4)",
        )
        .bind(session.id.as_uuid())
        .bind(session.call_id.as_uuid())
        .bind(&payload)
        .bind(session.started_at)
        .execute(&self.pool)
        .await
        .map_err(query_error)?;
        Ok(())
    }

    async fn update_session(&self, session: &Session) -> Result<(), StorageError> {
        let payload = serde_json::to_value(session).map_err(serde_error)?;
        sqlx::query("update sessions set payload = $2, ended_at = $3 where id = $1")
            .bind(session.id.as_uuid())
            .bind(&payload)
            .bind(session.ended_at)
            .execute(&self.pool)
            .await
            .map_err(query_error)?;
        Ok(())
    }

    async fn load_session(&self, id: SessionId) -> Result<Option<Session>, StorageError> {
        let row: Option<PgRow> = sqlx::query("select payload from sessions where id = $1")
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(query_error)?;
        row.as_ref().map(|r| decode(r, "payload")).transpose()
    }

    async fn insert_message(&self, message: &Message) -> Result<(), StorageError> {
        let payload = serde_json::to_value(message).map_err(serde_error)?;
        sqlx::query(
            "insert into messages (id, session_id, payload, created_at) values ($1, $2, $3, $4)",
        )
        .bind(message.id.as_uuid())
        .bind(message.session_id.as_uuid())
        .bind(&payload)
        .bind(message.created_at)
        .execute(&self.pool)
        .await
        .map_err(query_error)?;
        Ok(())
    }

    async fn load_messages(
        &self,
        session_id: SessionId,
        limit: Option<usize>,
    ) -> Result<Vec<Message>, StorageError> {
        let rows = sqlx::query(
            "select payload from messages where session_id = $1 order by created_at asc limit $2",
        )
        .bind(session_id.as_uuid())
        .bind(limit.unwrap_or(1_000) as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(query_error)?;
        decode_many(&rows, "payload")
    }

    async fn ping(&self) -> Result<(), StorageError> {
        sqlx::query("select 1")
            .execute(&self.pool)
            .await
            .map_err(query_error)?;
        Ok(())
    }
}
