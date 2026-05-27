use color_eyre::eyre::Report;
use datafusion::arrow::error::ArrowError;
use datafusion::error::DataFusionError;
use hyper::http::uri::{InvalidUri, InvalidUriParts};
use hyper::http::Error as HttpError;
use hyper::Error as HyperError;
use std::io::Error as IoError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadBalancerError {
    #[error("worker host address is empty")]
    EmptyWorkerHostAddress,

    #[error("no active worker found")]
    NoHealthyWorkers,

    #[error("invalid uri")]
    InvalidUri(#[from] InvalidUri),

    #[error("invalid uri parts")]
    InvalidUriParts(#[from] InvalidUriParts),

    #[error("arrow error")]
    Arrow(#[from] ArrowError),

    #[error("datafusion error")]
    DataFusion(#[from] DataFusionError),

    #[error("io error")]
    Io(#[from] IoError),

    #[error("http error")]
    Http(#[from] HttpError),

    #[error("hyper error")]
    Hyper(#[from] HyperError),

    #[error(transparent)]
    Unexpected(#[from] Report),
}
