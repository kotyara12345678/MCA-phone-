//! Shared Postgres helpers: error mapping and payload-column decoding used by
//! both halves of the store.

use mca_core::StorageError;
use serde_json::Value;
use sqlx::postgres::PgRow;
use sqlx::Row;

pub(crate) fn query_error(err: sqlx::Error) -> StorageError {
    StorageError::Query(err.to_string())
}

pub(crate) fn serde_error(err: serde_json::Error) -> StorageError {
    StorageError::Serde(err.to_string())
}

pub(crate) fn payload_error(err: sqlx::Error) -> StorageError {
    StorageError::Query(format!("cannot read payload column: {err}"))
}

pub(crate) fn decode<T>(row: &PgRow, column: &str) -> Result<T, StorageError>
where
    T: serde::de::DeserializeOwned,
{
    let payload: Value = row.try_get(column).map_err(payload_error)?;
    serde_json::from_value(payload).map_err(serde_error)
}

pub(crate) fn decode_many<T>(rows: &[PgRow], column: &str) -> Result<Vec<T>, StorageError>
where
    T: serde::de::DeserializeOwned,
{
    rows.iter().map(|row| decode(row, column)).collect()
}
