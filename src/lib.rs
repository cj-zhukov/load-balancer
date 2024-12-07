use std::sync::Arc;

use bytes::Bytes;
use hyper::{
    body::Incoming, Request, Response,
};
use tokio::sync::RwLock;

pub mod error;
pub mod data_store;
pub mod load_balancer;
pub mod utils;

use error::LoadBalancerError;
pub use load_balancer::LoadBalancer;

pub type BoxBody = http_body_util::combinators::BoxBody<Bytes, hyper::Error>;

pub async fn handler(
    req: Request<Incoming>, 
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<Response<BoxBody>, LoadBalancerError> {
    let mut load_balancer = load_balancer.write().await;
    load_balancer.forward_request(req).await
}