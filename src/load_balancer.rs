use std::str::FromStr;
use std::sync::Arc;

use color_eyre::eyre::{Context, ContextCompat};
use datafusion::arrow::array::{BooleanArray, Int64Array, RecordBatch};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::prelude::*;
use http_body_util::BodyExt;
use hyper::{
    client::conn::http1,
    body::Incoming, Request, Response, Uri
};
use hyper_util::rt::TokioIo;
use tokio::net::TcpStream;

use crate::data_store::Worker;
use crate::utils::{df_to_table, validate_address, DF_TABLE_NAME, HEALTH_ROUTE};
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

    pub async fn show_table(&self) -> Result<(), LoadBalancerError> {
        let df = self.ctx.sql(&format!("select * from {DF_TABLE_NAME} order by id")).await?;
        df.show().await?;
        Ok(())
    }
}

impl LoadBalancer {
    pub async fn forward_request(&mut self, req: Request<Incoming>) -> Result<Response<BoxBody>, LoadBalancerError> {
        self.check_workers_health().await?; // check all workers and setup status #TODO move it into init stage
        let mut worker_uri = self.get_worker().await?; // get active worker

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

    /// Check all workers and setup 'status'
    async fn check_workers_health(&self) -> Result<(), LoadBalancerError> {
        println!("checking worker servers health");
        let workers_df = self.ctx
            .sql(&format!("select id, worker_name, port_name, count_cons from {DF_TABLE_NAME}"))
            .await?;
        let workers = Worker::to_records(workers_df.clone())
            .await
            .map_err(|e| LoadBalancerError::UnexpectedError(e.into()))?;

        let mut tasks = vec![];
        for worker in workers {
            let worker_id = worker.id
                .wrap_err("worker port is empty")
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
            let worker_url_health = format!("{worker_url}{HEALTH_ROUTE}");
            let task = tokio::spawn(check_health(worker_id, worker_url_health));
            tasks.push(task);
        }

        let mut ids = vec![];
        let mut statuses = vec![];
        for task in tasks {
            match task.await {
                Ok(res) => {
                    ids.push(res.0);
                    statuses.push(res.1);
                },
                Err(e) => eprintln!("Failed running worker activity status: {e}")
            };
        }
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("status", DataType::Boolean, true),
        ]);
        let batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(Int64Array::from(ids)),
                Arc::new(BooleanArray::from(statuses)),
            ],
        )?;
        let workers_status = self.ctx
            .read_batch(batch)?
            .with_column_renamed("id", "id2")?;

        let active_workers = workers_df
            .join(workers_status, JoinType::Inner, &["id"], &["id2"], None)?
            .drop_columns(&["id2"])?;
        self.ctx.deregister_table(format!("{DF_TABLE_NAME}"))?;
        df_to_table(self.ctx.clone(), active_workers, DF_TABLE_NAME).await?;

        Ok(())
    }

    /// Get active worker 
    async fn get_worker(&mut self) -> Result<String, LoadBalancerError> {
        match self.algorithm {
            Algorithm::RoundRobin => {
                let cur_worker = self.current_worker;
                let round_robin_sql = format!("(({cur_worker} - 1) % (select count(*) from {DF_TABLE_NAME})) + 1");
                let df = self.ctx
                    .sql(&format!("select id, worker_name, port_name, count_cons 
                                 from {DF_TABLE_NAME} 
                                 where 1 = 1 
                                 and status is true
                                 and id = {round_robin_sql}"))
                    .await
                    .map_err(|e| LoadBalancerError::UnexpectedError(e.into()))?;

                let workers = Worker::to_records(df)
                    .await
                    .map_err(|e| LoadBalancerError::UnexpectedError(e.into()))?;

                if workers.is_empty() {
                    return Err(LoadBalancerError::EmptyWorkerHostAddress);
                }

                // check each worker if alive
                let mut idx = 0usize;
                let res = loop {
                    let worker = workers.get(idx)
                        .wrap_err("workers are empty")
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

                    let worker_url_health = format!("{worker_url}{HEALTH_ROUTE}");
                    if alive(worker_url_health).await {
                        self.current_worker += 1;
                        break worker_url;
                    }

                    idx += 1;
                };

                Ok(res)
            },

            Algorithm::LeastConnections => {
                let sql = format!("select id, worker_name, port_name, count_cons
                                            from {DF_TABLE_NAME} 
                                            where 1 = 1 
                                            and status is true
                                            and count_cons = (select min(count_cons) from {DF_TABLE_NAME})");
                let df = self.ctx
                    .sql(&sql)
                    .await
                    .map_err(|e| LoadBalancerError::UnexpectedError(e.into()))?;

                let workers = Worker::to_records(df)
                    .await
                    .wrap_err("error when parsing to records")
                    .map_err(|e| LoadBalancerError::UnexpectedError(e.into()))?;

                if workers.is_empty() {
                    return Err(LoadBalancerError::EmptyWorkerHostAddress);
                }

                // check each worker if alive
                let mut idx = 0usize;
                let res = loop {
                    let worker = workers.get(idx)
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

                    let worker_url_health = format!("{worker_url}{HEALTH_ROUTE}");
                    if alive(worker_url_health).await {
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
                        break worker_url;
                    }

                    idx += 1;
                };

                Ok(res)   
            }
        }
    }
}

pub async fn alive(url: String) -> bool {
    let client = reqwest::Client::new();

    client.get(url)
        .send()
        .await
        .ok()
        .map(|response| {
            let res = match response.status().as_u16() {
                200 => true,
                _ => false
            };
            Some(res)
        })
        .flatten()
        .unwrap_or(false)
}

pub async fn check_health(id: i64, url: String) -> (i64, bool) {
    let client = reqwest::Client::new();

    let status = client.get(url)
        .send()
        .await
        .ok()
        .map(|response| {
            let res = match response.status().as_u16() {
                200 => true,
                _ => false
            };
            Some(res)
        })
        .flatten()
        .unwrap_or(false);

    (id, status)
}