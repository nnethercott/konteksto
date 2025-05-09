use konteksto_engine::Args;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;

/// global settings for web server
pub struct Settings {
    pub server: ServerConfig,
    pub db: DbConfig,
    pub engine: Args,
}

/// config for connecting to sqlite db
pub struct DbConfig {
    pub file: String,
}
impl DbConfig {
    pub async fn create_pool(&self) -> sqlx::Result<SqlitePool> {
        let pool = SqlitePool::connect_with(
            SqliteConnectOptions::from_str(&format!("sqlite://{}", &self.file))?
                .create_if_missing(true),
        )
        .await?;

        // apply db migrations
        sqlx::migrate!("../migrations").run(&pool).await?;
        Ok(pool)
    }
}

/// settings specific to the web server
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}
impl ServerConfig {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
