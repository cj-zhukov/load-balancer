use color_eyre::eyre::Report;
use datafusion::error::DataFusionError;
use thiserror::Error;
use datafusion::arrow::error::ArrowError;
use sqlx::error::Error as DBError;
use sqlx::migrate::MigrateError;

#[derive(Debug, Error)]
pub enum DataStoreError {
    #[error("Dataframe is empty")]
    EmptyDataframeError,

    #[error("ArrowError")]
    ArrowError(#[from] ArrowError),

    #[error("DataFusionError")]
    DataFusionError(#[from] DataFusionError),

    #[error("SQLx db error")]
    DBError(#[from] DBError),

    #[error("MigrateError sqlx error")]
    MigrateError(#[from] MigrateError),
    
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}