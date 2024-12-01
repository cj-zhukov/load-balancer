use std::{net::SocketAddr, sync::Arc, error::Error};

use hyper_util::rt::TokioIo;
use hyper::service::service_fn;
use tokio::{net::TcpListener, sync::RwLock};
use hyper::server::conn::http1;

use load_balancer::{
    handler, LoadBalancer, WorkerHosts, 
    utils::{LOAD_BALANCER_NAME, LOAD_BALANCER_ADDRESS_SECRET, WORKERS_ADDRESSES_SECRET, parse_workers_addresses}
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install().expect("Failed to install color_eyre");

    let workers_addresses = parse_workers_addresses(&WORKERS_ADDRESSES_SECRET);
    let worker_hosts = WorkerHosts::new(workers_addresses)?;

    let load_balancer = Arc::new(RwLock::new(
        LoadBalancer::new(worker_hosts).expect("failed to create load balancer"),
    ));

    let addr = LOAD_BALANCER_ADDRESS_SECRET.parse::<SocketAddr>()?;
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("load balancer: {} is serving on port: {}", LOAD_BALANCER_NAME, addr.port());

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let load_balancer_clone = load_balancer.clone();

        tokio::task::spawn(async move {
            let service = service_fn(move |req| handler(req, load_balancer_clone.clone()));
            let http = http1::Builder::new();


            if let Err(err) = http.serve_connection(io, service).await {
                eprintln!("Failed to serve connection: {:?}", err);
            }
        });
    }
}