use std::sync::Arc;

use tokio::sync::RwLock;

use crate::{data_store::Table, domain::Database};

pub type DbConRef<T> = Arc<RwLock<DbConnector<T>>>;

pub struct DbConnector<T: Database + Send + Sync> {
    db: T,
}

impl<T: Database + Send + Sync> DbConnector<T> {
    pub fn new(db: T) -> Self {
        Self { db }
    }

    pub async fn query(&self, query: &str) -> Result<(), sqlx::Error> {
        self.db.execute_query(query).await
    }

    pub async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        self.db.run_migrations().await
    }

    pub async fn fetch_data(&self, query: &str) -> Result<Vec<Table>, sqlx::Error> {
        self.db.fetch_data(query).await
    }
}