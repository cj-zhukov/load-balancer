use std::sync::Arc;

use chrono::NaiveDateTime;
use datafusion::arrow::array::{BooleanArray, Int64Array, RecordBatch, StringArray};
use datafusion::prelude::*;
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::scalar::ScalarValue;
use serde_json::Value;
use serde::{Deserialize, Serialize};

// use crate::domain::Database;
// use crate::utils::{LOAD_BALANCER_NAME, TABLE_NAME};
// use crate::DbRef;
use super::DataStoreError;

#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct Table {
    pub id: i64,
    pub server_name: String,
    pub worker_name: String,
    pub port_name: String,
    pub active: bool,
    pub info: Option<Value>, 
    pub inserted_at: NaiveDateTime,
}

impl Table {
    pub fn schema() -> Schema {
        Schema::new(vec![
            Field::new("id", DataType::Int64, false),
            Field::new("server_name", DataType::Utf8, true),
            Field::new("worker_name", DataType::Utf8, true),
            Field::new("port_name", DataType::Utf8, true),
            Field::new("active", DataType::Boolean, true),
            Field::new("info", DataType::Utf8, true),
            Field::new("inserted_at", DataType::Utf8, true),
        ])
    }
}

impl Table {
    pub fn to_df(ctx: &SessionContext, records: &mut Vec<Self>) -> Result<DataFrame, DataStoreError> {
        let mut ids = Vec::new();
        let mut server_names = Vec::new();
        let mut worker_names = Vec::new();
        let mut port_names = Vec::new();
        let mut actives = Vec::new();
        let mut infos = Vec::new();
        let mut inserted_at_all = Vec::new();

        for record in records {
            ids.push(record.id);
            server_names.push(record.server_name.as_ref());
            worker_names.push(record.worker_name.as_ref());
            port_names.push(record.port_name.as_ref());
            actives.push(record.active);
            let info = match &mut record.info {
                Some(v) => {
                    Some(serde_json::to_string(&v).map_err(|e| DataStoreError::UnexpectedError(e.into()))?)
                }
                None => None
            };
            infos.push(info);
            inserted_at_all.push(record.inserted_at.to_string());
        }

        let schema = Self::schema();
        let batch = RecordBatch::try_new(
            schema.into(),
            vec![
                Arc::new(Int64Array::from(ids)), 
                Arc::new(StringArray::from(server_names)),
                Arc::new(StringArray::from(worker_names)),
                Arc::new(StringArray::from(port_names)),
                Arc::new(BooleanArray::from(actives)),
                Arc::new(StringArray::from(infos)),
                Arc::new(StringArray::from(inserted_at_all)),
            ],
        )?;
        let df = ctx.read_batch(batch)?;
        let df = df.with_column("count_cons", Expr::Literal(ScalarValue::Int64(Some(0))))?; // #TODO provide schema for this col

        Ok(df)
    }

    // pub async fn init_table<T>(ctx: SessionContext, db_ref: DbRef<T>) -> Result<DataFrame, DataStoreError> 
    // where
    //     T: Database + Send + Sync,
    // {
    //     let sql = format!("select * from {} 
    //                                 where 1 = 1
    //                                 and server_name = '{}' 
    //                                 and active is true", TABLE_NAME, LOAD_BALANCER_NAME);
    //     let db_con = db_ref.read().await;
    //     let mut records = db_con.fetch_data(&sql).await?;
    //     if records.is_empty() {
    //         return Err(DataStoreError::EmptyDataframeError);
    //     }
    //     let df = Self::to_df(ctx, &mut records)?;
    //     let df = df.with_column("count_cons", Expr::Literal(ScalarValue::Int64(Some(0))))?; // add count connections column with 0

    //     Ok(df)
    // }
}

