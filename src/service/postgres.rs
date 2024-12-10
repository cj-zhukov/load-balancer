use async_trait::async_trait;
use secrecy::{ExposeSecret, Secret};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

use crate::{data_store::Table, domain::Database};

pub struct PostgresDbBuilder {
    url: String,
    max_cons: u32,
}

impl Default for PostgresDbBuilder {
    fn default() -> Self {
        PostgresDbBuilder { url: String::default(), max_cons: 10 }
    }
}

impl PostgresDbBuilder {
    pub fn with_url(self, url: &str) -> Self {
        Self { url: url.to_string(), max_cons: self.max_cons }
    }

    pub fn with_max_cons(self, max_cons: u32) -> Self {
        Self { url: self.url, max_cons }
    }

    pub async fn build(self) -> Result<PostgresDb, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(self.max_cons)
            .connect(&self.url)
            .await?;

        Ok(PostgresDb { pool, url: Secret::new(self.url) })
    }
}

#[derive(Debug)]
pub struct PostgresDb {
    pool: Pool<Postgres>,
    url: Secret<String>,
}

impl AsRef<Pool<Postgres>> for PostgresDb {
    fn as_ref(&self) -> &Pool<Postgres> {
        &self.pool
    }
}

impl PostgresDb {
    pub fn builder() -> PostgresDbBuilder {
        PostgresDbBuilder::default()
    }

    pub fn get_url(&self) -> &str {
        self.url.expose_secret()
    }
}

#[async_trait]
impl Database for PostgresDb {
    async fn execute_query(&self, query: &str) -> Result<(), sqlx::Error> {
        let rows = sqlx::query(query)
            .fetch_all(self.as_ref())
            .await?;

        for row in rows {
            println!("{:?}", row);
        }

        Ok(())
    }

    async fn run_migrations(&self) -> Result<(), sqlx::Error> {
        sqlx::migrate!()
            .run(self.as_ref())
            .await?;

        println!("run migrations for server");

        Ok(())
    }

    async fn fetch_data(&self, query: &str) -> Result<Vec<Table>, sqlx::Error> {
        let query = sqlx::query_as::<_, Table>(query);
        let data = query.fetch_all(self.as_ref()).await?;

        Ok(data)
    }
}