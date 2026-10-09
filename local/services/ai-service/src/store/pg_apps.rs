//! Postgres implementation of the application/email half of the store.

use async_trait::async_trait;
use mca_core::domain::application_record::Application;
use mca_core::email::EmailDraft;
use mca_core::{ApplicationId, StorageError};
use sqlx::postgres::PgRow;
use sqlx::PgPool;

use super::pg_helpers::{decode, query_error, serde_error};
use super::ApplicationStore;

pub struct PostgresApplications {
    pool: PgPool,
}

impl PostgresApplications {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ApplicationStore for PostgresApplications {
    async fn insert_application(
        &self,
        application: &Application,
        email: Option<&EmailDraft>,
    ) -> Result<(), StorageError> {
        let payload = serde_json::to_value(application).map_err(serde_error)?;
        sqlx::query(
            "insert into applications (id, session_id, call_id, payload, created_at, updated_at)
             values ($1, $2, $3, $4, $5, $5)",
        )
        .bind(application.id.as_uuid())
        .bind(application.session_id.as_uuid())
        .bind(application.call_id.as_uuid())
        .bind(&payload)
        .bind(application.created_at)
        .execute(&self.pool)
        .await
        .map_err(query_error)?;
        if let Some(draft) = email {
            let email_payload = serde_json::to_value(draft).map_err(serde_error)?;
            sqlx::query(
                "insert into emails (id, application_id, to_address, subject, payload, generated_at)
                 values ($1, $2, $3, $4, $5, $6)",
            )
            .bind(draft.id.as_uuid())
            .bind(draft.application_id.as_uuid())
            .bind(&draft.to)
            .bind(&draft.subject)
            .bind(&email_payload)
            .bind(draft.generated_at)
            .execute(&self.pool)
            .await
            .map_err(query_error)?;
        }
        Ok(())
    }

    async fn load_application(
        &self,
        id: ApplicationId,
    ) -> Result<Option<Application>, StorageError> {
        let row: Option<PgRow> = sqlx::query("select payload from applications where id = $1")
            .bind(id.as_uuid())
            .fetch_optional(&self.pool)
            .await
            .map_err(query_error)?;
        match row {
            Some(row) => decode(&row, "payload").map(Some),
            None => Ok(None),
        }
    }

    async fn load_email(
        &self,
        application_id: ApplicationId,
    ) -> Result<Option<EmailDraft>, StorageError> {
        let row: Option<PgRow> =
            sqlx::query("select payload from emails where application_id = $1")
                .bind(application_id.as_uuid())
                .fetch_optional(&self.pool)
                .await
                .map_err(query_error)?;
        match row {
            Some(row) => decode(&row, "payload").map(Some),
            None => Ok(None),
        }
    }
}
