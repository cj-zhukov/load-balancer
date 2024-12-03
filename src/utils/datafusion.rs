use std::sync::Arc;

use datafusion::datasource::MemTable;
use datafusion::prelude::*;
use datafusion::error::DataFusionError;

/// Register dataframe as table in ctx
pub async fn df_to_table(ctx: SessionContext, df: DataFrame, table_name: &str) -> Result<(), DataFusionError> {
    let schema = df.clone().schema().as_arrow().clone();
    let batches = df.collect().await?;
    let mem_table = MemTable::try_new(Arc::new(schema), vec![batches])?;
    ctx.register_table(table_name, Arc::new(mem_table))?;

    Ok(())
}

/// Check if dataframe is empty
pub async fn is_empty(df: DataFrame) -> Result<bool, DataFusionError> {
    let batches = df.collect().await?;
    let is_empty = batches.iter().all(|batch| batch.num_rows() == 0);
    
    Ok(is_empty)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use datafusion::{arrow::{array::{Int32Array, RecordBatch, StringArray}, datatypes::{DataType, Field, Schema}}, assert_batches_eq, prelude::*};
    use super::*;
    
    #[tokio::test]
    async fn test_df_to_table() {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("data", DataType::Int32, true),
            Field::new("name", DataType::Utf8, true),

        ]);
        let batch = RecordBatch::try_new(
            schema.clone().into(),
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3])),
                Arc::new(Int32Array::from(vec![42, 43, 44])),
                Arc::new(StringArray::from(vec!["foo", "bar", "baz"])),
            ],
        ).unwrap();

        let ctx = SessionContext::new();
        let df = ctx.read_batch(batch).unwrap();
        df_to_table(ctx.clone(), df, "t").await.unwrap();
        let res = ctx.sql("select * from t order by id").await.unwrap();
        assert_batches_eq!(
            &[
                "+----+------+------+",
                "| id | data | name |",
                "+----+------+------+",
                "| 1  | 42   | foo  |",
                "| 2  | 43   | bar  |",
                "| 3  | 44   | baz  |",
                "+----+------+------+",
            ],
            &res.collect().await.unwrap()
        );
    }

    #[tokio::test]
    async fn test_is_empty() {
        let schema = Schema::new(vec![
            Field::new("id", DataType::Int32, false),
            Field::new("name", DataType::Utf8, true),
            Field::new("data", DataType::Int32, true),
        ]);
        let batch = RecordBatch::try_new(
            schema.clone().into(),
            vec![
                Arc::new(Int32Array::from(vec![1, 2, 3])),
                Arc::new(StringArray::from(vec!["foo", "bar", "baz"])),
                Arc::new(Int32Array::from(vec![42, 43, 44])),
            ],
        ).unwrap();

        let ctx = SessionContext::new();
        let df = ctx.read_batch(batch).unwrap();
        assert_eq!(is_empty(df).await.unwrap(), false);

        let batch = RecordBatch::new_empty(schema.into());
        let df = ctx.read_batch(batch).unwrap();
        assert_eq!(is_empty(df).await.unwrap(), true);
    }
}