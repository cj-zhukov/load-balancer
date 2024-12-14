use color_eyre::eyre::Report;
use datafusion::arrow::error::ArrowError;
use datafusion::error::DataFusionError;
use thiserror::Error;
use hyper::http::Error as HttpError;
use hyper::http::uri::InvalidUri;
use hyper::Error as HyperError;
use std::io::Error as IoError;
use regex::Error as RegexError;
use reqwest::Error as ReqwestError;

#[derive(Debug, Error)]
pub enum LoadBalancerError {
    #[error("Invalid worker hosts address")]
    InvalidWorkerHostAddress,

    #[error("Empty worker hosts address")]
    EmptyWorkerHostAddress,

    #[error("Invalid load balancer address")]
    InvalidLoadBalancerAddress,

    #[error("Invalid load balancer current worker")]
    InvalidLoadBalancerCurrentWorker,

    #[error("Invalid uri")]
    InvalidUri(#[from] InvalidUri),

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

    #[error("Regex error")]
    RegexError(#[from] RegexError),

    #[error("Reqwest error")]
    ReqwestError(#[from] ReqwestError),
    
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}