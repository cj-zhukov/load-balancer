use std::str::FromStr;

use color_eyre::eyre::ContextCompat;
use http_body_util::BodyExt;
use hyper::{
    client::conn::http1,
    body::Incoming, Request, Response, Uri
};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

use crate::BoxBody;
use crate::error::LoadBalancerError;
use super::WorkerHosts;

pub struct LoadBalancer {
    pub worker_hosts: WorkerHosts,
    pub current_worker: usize,
}

impl LoadBalancer {
    pub fn new(worker_hosts: WorkerHosts) -> Result<Self, LoadBalancerError> {
        Ok(LoadBalancer {
            worker_hosts,
            current_worker: 0,
        })
    }

    pub async fn forward_request(&mut self, req: Request<Incoming>) -> Result<Response<BoxBody>, LoadBalancerError> {
        let mut worker_uri = self.get_worker()?.to_string();

        // Extract the path and query from the original request
        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
        }

        // Create a new URI from the worker URI
        let new_uri = Uri::from_str(&worker_uri).map_err(LoadBalancerError::InvalidUri)?;
        let new_host = new_uri
            .host()
            .wrap_err(format!("failed parsing uri: {new_uri} to get host"))
            .map_err(|e| LoadBalancerError::UnexpectedError(e))?;
        let new_port = new_uri
            .port()
            .wrap_err(format!("failed parsing uri: {new_uri} to get port"))
            .map_err(|e| LoadBalancerError::UnexpectedError(e))?;
        let new_address = format!("{new_host}:{new_port}");

        // Extract the headers from the original request
        let headers = req.headers().clone();

        // Clone the original request's headers and method
        let mut new_req = Request::builder()
            .method(req.method())
            .uri(&new_uri)
            .body(req.into_body())
            .map_err(LoadBalancerError::HttpError)?;

        // Copy headers from the original request
        for (key, value) in headers.iter() {
            new_req.headers_mut().insert(key, value.clone());
        }

        // Sending new request to worker
        let client_stream = TcpStream::connect(new_address).await?;
        let io = TokioIo::new(client_stream);
        let (mut sender, conn) = http1::handshake(io).await?;
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                eprintln!("Connection failed: {:?}", err);
            }
        });

        let worker_res = sender.send_request(new_req).await?;
        let res_body = worker_res.into_body().boxed();

        Ok(Response::new(res_body))
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