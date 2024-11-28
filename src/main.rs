use std::{convert::Infallible, net::SocketAddr, sync::Arc, error::Error};

use hyper::{service::{make_service_fn, service_fn}, Server};
use tokio::sync::RwLock;

use load_balancer::{
    handle, LoadBalancer, WorkerHosts, 
    utils::{LOAD_BALANCER_NAME, LOAD_BALANCER_ADDRESS_SECRET, WORKERS_ADDRESSES_SECRET, parse_workers_addresses}
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install().expect("Failed to install color_eyre");
    let addresses = parse_workers_addresses(&WORKERS_ADDRESSES_SECRET);
    let worker_hosts = WorkerHosts::new(addresses)?;

    let load_balancer = Arc::new(RwLock::new(
        LoadBalancer::new(worker_hosts).expect("failed to create load balancer"),
    ));

    let addr = LOAD_BALANCER_ADDRESS_SECRET.parse::<SocketAddr>()?;

    let server = Server::bind(&addr).serve(make_service_fn(move |_conn| {
        let load_balancer = load_balancer.clone();
        async move { Ok::<_, Infallible>(service_fn(move |req| handle(req, load_balancer.clone()))) }
    }));

    println!("load balancer: {} is serving on port: {}", LOAD_BALANCER_NAME, addr.port());

    if let Err(e) = server.await {
        eprintln!("error: {}", e);
    }

    Ok(())
}