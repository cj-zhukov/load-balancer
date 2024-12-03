use std::{net::SocketAddr, sync::Arc, error::Error};

use datafusion::prelude::SessionContext;
use hyper_util::rt::TokioIo;
use hyper::service::service_fn;
use tokio::{net::TcpListener, sync::RwLock};
use hyper::server::conn::http1;

use load_balancer::{
    data_store::{Table, DB}, 
    handler, 
    utils::{df_to_table, DB_ADDRESS, DB_NAME_SECRET, DB_PASSWORD_SECRET, DB_USER_SECRET, DF_TABLE_NAME, LOAD_BALANCER_ADDRESS_SECRET, LOAD_BALANCER_NAME, MAX_DB_CONNECTIONS}, 
    LoadBalancer, 
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install().expect("Failed to install color_eyre");

    let db = DB::build(DB_ADDRESS, &DB_USER_SECRET, &DB_PASSWORD_SECRET, &DB_NAME_SECRET, MAX_DB_CONNECTIONS).await?;
    db.run_migrations().await?;

    let ctx = SessionContext::new();
    let worker_hosts = Table::init_table(ctx.clone(), db.server).await?; // fetch and store worker hosts as df
    df_to_table(ctx.clone(), worker_hosts.clone(), DF_TABLE_NAME).await?; // register table in ctx

    let load_balancer = Arc::new(RwLock::new(
        LoadBalancer::new(ctx, Some(1)).expect("failed to create load balancer"),
    ));

    let addr = LOAD_BALANCER_ADDRESS_SECRET.parse::<SocketAddr>()?;
    let listener = TcpListener::bind(addr).await?;
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