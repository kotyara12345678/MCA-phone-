//! In-memory store, used by integration tests (and `--memory` local runs).
//!
//! The dialogue engine only depends on the store traits, so swapping Postgres
//! for this map keeps every deterministic E2E test independent of Docker.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use mca_core::domain::application_record::Application;
use mca_core::domain::message::Message;
use mca_core::domain::session::Session;
use mca_core::email::EmailDraft;
use mca_core::{ApplicationId, SessionId, StorageError};

use super::SessionStore;

#[derive(Debug, Default)]
pub struct MemStore {
    pub(crate) sessions: Mutex<HashMap<SessionId, Session>>,
    pub(crate) messages: Mutex<HashMap<SessionId, Vec<Message>>>,
    pub(crate) applications: Mutex<HashMap<ApplicationId, Application>>,
    pub(crate) emails: Mutex<HashMap<ApplicationId, EmailDraft>>,
}

impl MemStore {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl SessionStore for MemStore {
    async fn insert_session(&self, session: &Session) -> Result<(), StorageError> {
        self.sessions
            .lock()
            .map_err(|_| StorageError::Query("mem store lock poisoned".into()))?
            .insert(session.id, session.clone());
        Ok(())
    }

    async fn update_session(&self, session: &Session) -> Result<(), StorageError> {
        self.insert_session(session).await
    }

    async fn load_session(&self, id: SessionId) -> Result<Option<Session>, StorageError> {
        Ok(self
            .sessions
            .lock()
            .map_err(|_| StorageError::Query("mem store lock poisoned".into()))?
            .get(&id)
            .cloned())
    }

    async fn insert_message(&self, message: &Message) -> Result<(), StorageError> {
        self.messages
            .lock()
            .map_err(|_| StorageError::Query("mem store lock poisoned".into()))?
            .entry(message.session_id)
            .or_default()
            .push(message.clone());
        Ok(())
    }

    async fn load_messages(
        &self,
        session_id: SessionId,
        limit: Option<usize>,
    ) -> Result<Vec<Message>, StorageError> {
        let guard = self
            .messages
            .lock()
            .map_err(|_| StorageError::Query("mem store lock poisoned".into()))?;
        let list = guard.get(&session_id).cloned().unwrap_or_default();
        Ok(match limit {
            Some(n) => list.into_iter().take(n).collect(),
            None => list,
        })
    }

    async fn ping(&self) -> Result<(), StorageError> {
        Ok(())
    }
}
