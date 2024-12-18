use std::{net::SocketAddr, sync::Arc, error::Error};

use datafusion::prelude::SessionContext;
use hyper_util::rt::TokioIo;
use hyper::service::service_fn;
use secrecy::ExposeSecret;
use tokio::{net::TcpListener, sync::RwLock};
use hyper::server::conn::http1;

use load_balancer::{
    data_store::Table, 
    domain::Database, 
    handler, 
    load_balancer::Algorithm, 
    service::{PostgresDb, SqliteDb}, 
    utils::{df_to_table, DF_TABLE_NAME, LOAD_BALANCER_ADDRESS_SECRET, LOAD_BALANCER_NAME, MAX_DB_CONS, PG_DATABASE_URL, SQLITE_DATABASE_URL, TABLE_NAME}, 
    LoadBalancer 
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    color_eyre::install().expect("Failed to install color_eyre");

    let db = PostgresDb::builder()
        .with_url(PG_DATABASE_URL.expose_secret())
        .with_max_cons(MAX_DB_CONS)
        .build()
        .await?;
    db.run_migrations().await?;

    let sql = format!("select * from {TABLE_NAME} 
                                where 1 = 1
                                and server_name = '{LOAD_BALANCER_NAME}' 
                                and active is true");
    let mut records = db.fetch_data(&sql).await?;
    let ctx = SessionContext::new();
    let workers = Table::to_df(&ctx, &mut records)?;
    df_to_table(&ctx, workers, DF_TABLE_NAME).await?; 

    let mut load_balancer = LoadBalancer::new(ctx, 1);
    load_balancer.with_algorithm(Algorithm::Random);
    load_balancer.check_workers_health().await?;
    let load_balancer = Arc::new(RwLock::new(load_balancer));

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