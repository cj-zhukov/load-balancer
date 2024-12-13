use std::str::FromStr;

use color_eyre::eyre::{Context, ContextCompat};
use datafusion::prelude::*;
use http_body_util::BodyExt;
use hyper::{
    client::conn::http1,
    body::Incoming, Request, Response, Uri
};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

use crate::data_store::Worker;
use crate::utils::{df_to_table, validate_address, DF_TABLE_NAME};
use crate::BoxBody;
use crate::error::LoadBalancerError;

#[derive(Default)]
pub struct LoadBalancer {
    pub ctx: SessionContext,
    pub current_worker: usize,
    pub algorithm: Algorithm,
}

#[derive(Default)]
pub enum Algorithm {
    #[default]
    RoundRobin, // Distribute requests evenly across all worker servers
    LeastConnections, // Route requests to the worker server with the least active connections.
}

impl LoadBalancer {
    pub fn new(ctx: SessionContext, current_worker: usize) -> Self {
        LoadBalancer {
            ctx,
            current_worker,
            ..Default::default()
        }
    }

    pub fn with_algorithm(&mut self, algorithm: Algorithm) {
        self.algorithm = algorithm
    }

    pub async fn forward_request(&mut self, req: Request<Incoming>) -> Result<Response<BoxBody>, LoadBalancerError> {
        let mut worker_uri = self.get_worker().await?;

        if let Some(path_and_query) = req.uri().path_and_query() {
            worker_uri.push_str(path_and_query.as_str());
        }

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

        let headers = req.headers().clone();

        let mut new_req = Request::builder()
            .method(req.method())
            .uri(&new_uri)
            .body(req.into_body())
            .map_err(LoadBalancerError::HttpError)?;

        for (key, value) in headers.iter() {
            new_req.headers_mut().insert(key, value.clone());
        }

        println!("sending new request to worker: {}", new_address);
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

    async fn get_worker(&mut self) -> Result<String, LoadBalancerError> {
        match self.algorithm {
            Algorithm::RoundRobin => {
                let cur_worker = self.current_worker;
                let round_robin_sql = format!("(({cur_worker} - 1) % (select count(*) from {DF_TABLE_NAME})) + 1");
                let df = self.ctx
                    .sql(&format!("select id, worker_name, port_name, count_cons 
                                 from {DF_TABLE_NAME} 
                                 where id = {round_robin_sql}"))
                    .await
                    .map_err(|e| LoadBalancerError::UnexpectedError(e.into()))?;
                let workers = Worker::to_records(df)
                    .await
                    .map_err(|e| LoadBalancerError::UnexpectedError(e.into()))?;
                let worker = workers.get(0)
                    .wrap_err("workers is empty")
                    .map_err(|e| LoadBalancerError::UnexpectedError(e))?;
                let worker_name = worker.name.clone()
                    .wrap_err("worker name is empty")
                    .map_err(|e| LoadBalancerError::UnexpectedError(e))?;
                let worker_port = worker.port.clone()
                    .wrap_err("worker port is empty")
                    .map_err(|e| LoadBalancerError::UnexpectedError(e))?;
                let worker_url = format!("http://{}:{}", worker_name, worker_port);
                if !validate_address(&worker_url)? {
                    return Err(LoadBalancerError::InvalidWorkerHostAddress);
                }
                self.current_worker += 1;
        
                Ok(worker_url)
            },
            Algorithm::LeastConnections => {
                let sql = format!("select id, worker_name, port_name, count_cons
                                            from {DF_TABLE_NAME} 
                                            where count_cons = (select min(count_cons) from {DF_TABLE_NAME})");
                let df = self.ctx
                    .sql(&sql)
                    .await
                    .map_err(|e| LoadBalancerError::UnexpectedError(e.into()))?;
                let workers = Worker::to_records(df)
                    .await
                    .wrap_err("error when parsing to records")
                    .map_err(|e| LoadBalancerError::UnexpectedError(e.into()))?;
                let worker = workers.get(0)
                    .wrap_err("workers is empty")
                    .map_err(|e| LoadBalancerError::UnexpectedError(e))?;
                let worker_name = worker.name.clone()
                    .wrap_err("worker name is empty")
                    .map_err(|e| LoadBalancerError::UnexpectedError(e))?;
                let worker_port = worker.port.clone()
                    .wrap_err("worker port is empty")
                    .map_err(|e| LoadBalancerError::UnexpectedError(e))?;
                let worker_id = worker.id
                    .wrap_err("worker port is empty")
                    .map_err(|e| LoadBalancerError::UnexpectedError(e))?;
                let worker_url = format!("http://{}:{}", worker_name, worker_port);
                if !validate_address(&worker_url)? {
                    return Err(LoadBalancerError::InvalidWorkerHostAddress);
                }

                // updating count_cons column and register new table 
                let sql = format!("select id, worker_name, port_name, 
                                            case
                                                when id = {worker_id} then count_cons + 1
                                                else count_cons
                                            end as count_cons
                                            from {DF_TABLE_NAME}");
                let df = self.ctx.sql(&sql).await?;
                self.ctx.deregister_table(format!("{DF_TABLE_NAME}"))?;
                df_to_table(self.ctx.clone(), df, DF_TABLE_NAME).await?;
                // self.ctx.sql(&format!("select * from {DF_TABLE_NAME}")).await?.show().await?; // debug updated col count_cons

                Ok(worker_url)
            }
        }
    }
}