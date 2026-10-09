//! In-memory application/email half of the store. See `mem.rs` for the session
//! side; both share the single `MemStore` aggregate.

use async_trait::async_trait;
use mca_core::domain::application_record::Application;
use mca_core::email::EmailDraft;
use mca_core::{ApplicationId, StorageError};

use crate::store::mem::MemStore;
use crate::store::ApplicationStore;

#[async_trait]
impl ApplicationStore for MemStore {
    async fn insert_application(
        &self,
        application: &Application,
        email: Option<&EmailDraft>,
    ) -> Result<(), StorageError> {
        self.applications
            .lock()
            .map_err(|_| StorageError::Query("mem store lock poisoned".into()))?
            .insert(application.id, application.clone());
        if let Some(draft) = email {
            self.emails
                .lock()
                .map_err(|_| StorageError::Query("mem store lock poisoned".into()))?
                .insert(draft.application_id, draft.clone());
        }
        Ok(())
    }

    async fn load_application(
        &self,
        id: ApplicationId,
    ) -> Result<Option<Application>, StorageError> {
        Ok(self
            .applications
            .lock()
            .map_err(|_| StorageError::Query("mem store lock poisoned".into()))?
            .get(&id)
            .cloned())
    }

    async fn load_email(
        &self,
        application_id: ApplicationId,
    ) -> Result<Option<EmailDraft>, StorageError> {
        Ok(self
            .emails
            .lock()
            .map_err(|_| StorageError::Query("mem store lock poisoned".into()))?
            .get(&application_id)
            .cloned())
    }
}
