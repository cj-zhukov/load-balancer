use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use super::error::DataStoreError;

pub struct DB {
    pub server: Pool<Postgres>,
    pub address: String
}

impl DB {
    fn new(server: Pool<Postgres>, address: String) -> Self {
        Self { server, address }
    }

    pub async fn run_migrations(&self) -> Result<(), DataStoreError> {
        sqlx::migrate!().run(&self.server).await?;
        println!("run migrations for server {}", &self.address);

        Ok(())
    }

    pub async fn build(address: &str, user: &str, pwd: &str, db: &str, max_connections: u32) -> Result<Self, DataStoreError> {
        let url = format!("postgres://{}:{}@{}/{}", user, pwd, address, db);
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(&url)
            .await?;

        println!("established connection to server: {} db: {}", address, db);

        Ok(DB::new(pool, address.to_string()))
    }
}