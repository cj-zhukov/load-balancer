use color_eyre::eyre::Report;
use datafusion::arrow::error::ArrowError;
use datafusion::error::DataFusionError;
use thiserror::Error;
use hyper::http::Error as HttpError;
use hyper::http::uri::InvalidUri;
use hyper::http::uri::InvalidUriParts;
use hyper::Error as HyperError;
use std::io::Error as IoError;

#[derive(Debug, Error)]
pub enum LoadBalancerError {
    #[error("Empty worker hosts address")]
    EmptyWorkerHostAddress,

    #[error("Invalid uri")]
    InvalidUri(#[from] InvalidUri),

    #[error("InvalidUriParts error")]
    InvalidUriParts(#[from] InvalidUriParts),

    #[error("ArrowError")]
    ArrowError(#[from] ArrowError),

    #[error("DataFusionError")]
    DataFusionError(#[from] DataFusionError),

    #[error("IO error")]
    IoError(#[from] IoError),

    #[error("HTTP connections error")]
    HttpError(#[from] HttpError),

    #[error("Hyper error")]
    HyperError(#[from] HyperError),
    
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}