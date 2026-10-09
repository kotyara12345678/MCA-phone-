//! Free-function client RPCs for ai-service.
//!
//! Each function takes the gateway by reference so the tonic
//! trait implementation stays a three-line delegator — the same split that
//! keeps the server side inside one `impl`.

pub(crate) mod finish;
pub(crate) mod start;
pub(crate) mod turn;

pub(crate) use finish::finish;
pub(crate) use start::start;
pub(crate) use turn::turn;
