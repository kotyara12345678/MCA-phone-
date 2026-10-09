//! Persistence boundary for the dialogue engine.
//!
//! Two implementations exist: [`pg::PostgresStore`]/[`pg_apps::PostgresApplications`]
//! for production and [`mem::MemStore`] for tests and local runs. The split
//! mirrors the two aggregates the engine owns — a session plus its transcript,
//! and the finished application plus its email draft.

use async_trait::async_trait;
use mca_core::domain::application_record::Application;
use mca_core::domain::message::Message;
use mca_core::domain::session::Session;
use mca_core::email::EmailDraft;
use mca_core::{ApplicationId, SessionId, StorageError};

pub mod mem;
pub mod mem_apps;
pub mod pg;
pub mod pg_apps;
pub mod pg_helpers;

/// Session aggregate + transcript persistence.
#[async_trait]
pub trait SessionStore: Send + Sync {
    async fn insert_session(&self, session: &Session) -> Result<(), StorageError>;
    async fn update_session(&self, session: &Session) -> Result<(), StorageError>;
    async fn load_session(&self, id: SessionId) -> Result<Option<Session>, StorageError>;
    async fn insert_message(&self, message: &Message) -> Result<(), StorageError>;
    async fn load_messages(
        &self,
        session_id: SessionId,
        limit: Option<usize>,
    ) -> Result<Vec<Message>, StorageError>;
    /// Liveness probe for `/ready`.
    async fn ping(&self) -> Result<(), StorageError>;
}

/// Finished application + email persistence.
#[async_trait]
pub trait ApplicationStore: Send + Sync {
    async fn insert_application(
        &self,
        application: &Application,
        email: Option<&EmailDraft>,
    ) -> Result<(), StorageError>;
    async fn load_application(
        &self,
        id: ApplicationId,
    ) -> Result<Option<Application>, StorageError>;
    async fn load_email(
        &self,
        application_id: ApplicationId,
    ) -> Result<Option<EmailDraft>, StorageError>;
}
