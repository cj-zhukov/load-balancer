use std::sync::Arc;

use error::LoadBalancerError;
use hyper::{
    Body, Request, Response,
};
use tokio::sync::RwLock;

pub mod load_balancer;
pub mod error;
pub mod utils;

pub use load_balancer::{LoadBalancer, WorkerHosts};

pub async fn handle(
    req: Request<Body>, 
    load_balancer: Arc<RwLock<LoadBalancer>>,
) -> Result<Response<Body>, LoadBalancerError> {
    let mut load_balancer = load_balancer.write().await;
    let res = load_balancer.forward_request(req)?;
    Ok(res.await?)
}