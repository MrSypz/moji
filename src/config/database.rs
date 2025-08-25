use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use std::env;
use std::time::Duration;

/// Creates and returns a MySQL connection pool
pub async fn create_pool() -> Result<MySqlPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL environment variable must be set");

    // Configure connection pool with reasonable defaults
    MySqlPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .max_lifetime(Duration::from_secs(1800)) // 30 minutes
        .acquire_timeout(Duration::from_secs(30))
        .idle_timeout(Duration::from_secs(600)) // 10 minutes
        .connect(&database_url)
        .await
}