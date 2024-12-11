use datafusion::prelude::*;
use datafusion::arrow::array::AsArray;
use datafusion::arrow::datatypes::Int64Type;
use itertools::izip;
use tokio_stream::StreamExt;

use super::DataStoreError;

#[derive(Debug)]
pub struct Worker {
    pub id: Option<i64>,
    pub name: Option<String>, 
    pub port: Option<String>,
}

impl Worker {
    pub async fn to_records(df: DataFrame) -> Result<Vec<Self>, DataStoreError> {
        let mut stream = df.execute_stream().await?;
        let mut records = vec![];
        while let Some(batch) = stream.next().await.transpose()? {
            let ids = batch.column(0).as_primitive::<Int64Type>();
            let names = batch.column(1).as_string::<i32>();
            let ports = batch.column(2).as_string::<i32>();

            for (id, name, port) in izip!(ids, names, ports) {
                let name = name.map(|x| x.to_string());
                let port = port.map(|x| x.to_string());
                records.push(Worker {
                    id,
                    name,
                    port,
                });
            }
        }

        Ok(records)
    }
}