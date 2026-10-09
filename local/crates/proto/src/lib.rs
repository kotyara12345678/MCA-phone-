//! Shared gRPC contracts for the MCA Logistics voice AI platform.
//!
//! Generated code is committed under `OUT_DIR` by `tonic-prost-build`; the
//! service crates must never hand-write protobuf models. Import them via
//! `mca_proto::ai::v1`, `mca_proto::voice::v1`, `mca_proto::health::v1`.

pub mod ai {
    pub mod v1 {
        tonic::include_proto!("mca.ai.v1");
    }
}

pub mod voice {
    pub mod v1 {
        tonic::include_proto!("mca.voice.v1");
    }
}

pub mod health {
    pub mod v1 {
        tonic::include_proto!("grpc.health.v1");
    }
}

pub mod clock;

pub use ai::v1 as ai_v1;
pub use health::v1 as health_v1;
pub use voice::v1 as voice_v1;
