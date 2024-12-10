use async_trait::async_trait;

use crate::data_store::Table;

#[async_trait]
pub trait Database {
    async fn run_migrations(&self) -> Result<(), sqlx::Error>;
    async fn execute_query(&self, query: &str) -> Result<(), sqlx::Error>;
    async fn fetch_data(&self, query: &str) -> Result<Vec<Table>, sqlx::Error>;
}