pub mod models;
pub mod queries;
pub mod schema;

use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;

        let db = Self { pool };
        db.init_schema().await?;

        Ok(db)
    }

    async fn init_schema(&self) -> Result<()> {
        schema::create_tables(&self.pool).await?;
        Ok(())
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
