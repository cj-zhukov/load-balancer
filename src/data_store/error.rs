use color_eyre::eyre::Report;
use datafusion::arrow::error::ArrowError;
use datafusion::error::DataFusionError;
use sqlx::error::Error as SqlxError;
use sqlx::migrate::MigrateError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataStoreError {
    #[error("dataframe is empty")]
    EmptyDataframe,

    #[error("arrow error")]
    Arrow(#[from] ArrowError),

    #[error("datafusion error")]
    DataFusion(#[from] DataFusionError),

    #[error("database error")]
    Database(#[from] SqlxError),

    #[error("database migration error")]
    Migration(#[from] MigrateError),

    #[error(transparent)]
    Unexpected(#[from] Report),
}
