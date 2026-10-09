use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::ProviderError;

/// A domain event worth publishing. Typed and closed: no free-form payloads, so
/// adding a broker later cannot smuggle unstructured data across services.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DomainEvent {
    SessionStarted {
        session_id: String,
    },
    CallFinished {
        session_id: String,
        qualified: bool,
    },
    QualificationChanged {
        session_id: String,
        qualification: String,
    },
    ApplicationCreated {
        session_id: String,
        application_id: String,
    },
    /// A provider is degraded; the dialogue engine has fallen back to rules.
    ProviderDegraded {
        provider: String,
        error_kind: String,
    },
}

impl DomainEvent {
    /// Routing key. One queue per kind keeps ordering local and makes the
    /// optional RabbitMQ adapter a trivial mapping.
    pub fn routing_key(&self) -> &'static str {
        match self {
            Self::SessionStarted { .. } => "mca.session.started",
            Self::CallFinished { .. } => "mca.call.finished",
            Self::QualificationChanged { .. } => "mca.qualification.changed",
            Self::ApplicationCreated { .. } => "mca.application.created",
            Self::ProviderDegraded { .. } => "mca.provider.degraded",
        }
    }

    pub fn session_id(&self) -> &str {
        match self {
            Self::SessionStarted { session_id }
            | Self::CallFinished { session_id, .. }
            | Self::QualificationChanged { session_id, .. }
            | Self::ApplicationCreated { session_id, .. } => session_id,
            Self::ProviderDegraded { .. } => "-",
        }
    }
}

/// Publishing port. The MVP default is `LoggingBroker` (tracing only);
/// `RabbitMqBroker` is an optional adapter behind the `rabbitmq` compose
/// profile. Losing an event must never fail a call, so publish errors are
/// reported but not propagated to the dialogue engine.
#[async_trait]
pub trait EventBroker: Send + Sync {
    async fn publish(&self, event: DomainEvent) -> Result<(), ProviderError>;

    fn name(&self) -> &'static str;
}
