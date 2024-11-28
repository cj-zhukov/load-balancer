use color_eyre::eyre::Report;
use thiserror::Error;
use hyper::http::Error as HttpError;
use hyper::http::uri::InvalidUri;
use hyper::Error as HyperError;
use regex::Error as RegexError;

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

    #[error("HTTP connections error")]
    HttpError(#[from] HttpError),

    #[error("Hyper error")]
    HyperError(#[from] HyperError),

    #[error("Regex error")]
    RegexError(#[from] RegexError),
    
    #[error("Unexpected error")]
    UnexpectedError(#[source] Report),
}