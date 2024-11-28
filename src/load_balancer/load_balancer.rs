use std::str::FromStr;

use hyper::{
    client::{ResponseFuture, HttpConnector},
    Body, Client, Request, Uri,
};

use crate::error::LoadBalancerError;
use super::WorkerHosts;

pub struct LoadBalancer {
    pub client: Client<HttpConnector>,
    pub worker_hosts: WorkerHosts,
    pub current_worker: usize,
}

impl LoadBalancer {
    pub fn new(worker_hosts: WorkerHosts) -> Result<Self, LoadBalancerError> {
        let client = Client::new();

        Ok(LoadBalancer {
            client,
            worker_hosts,
            current_worker: 0,
        })
    }

    pub fn forward_request(&mut self, req: Request<Body>) -> Result<ResponseFuture, LoadBalancerError> {
        let mut worker_uri = self.get_worker()?.to_owned();

        // Extract the path and query from the original request
        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
        }

        // Create a new URI from the worker URI
        let new_uri = Uri::from_str(&worker_uri).map_err(|e| LoadBalancerError::InvalidUri(e))?;

        // Extract the headers from the original request
        let headers = req.headers().clone();

        // Clone the original request's headers and method
        let mut new_req = Request::builder()
            .method(req.method())
            .uri(new_uri)
            .body(req.into_body())
            .map_err(|e| LoadBalancerError::HttpError(e))?;

        // Copy headers from the original request
        for (key, value) in headers.iter() {
            new_req.headers_mut().insert(key, value.clone());
        }

        Ok(self.client.request(new_req))
    }

    fn get_worker(&mut self) -> Result<&str, LoadBalancerError> {
        // Use a round-robin strategy to select a worker
        let worker = self.worker_hosts.hosts.get(self.current_worker);
        if worker.is_none() {
            return Err(LoadBalancerError::InvalidLoadBalancerCurrentWorker);
        }

        self.current_worker = (self.current_worker + 1) % self.worker_hosts.hosts.len();

        Ok(worker.unwrap())
    }
}