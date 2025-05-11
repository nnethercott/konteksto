use konteksto_engine::Args;
use clap::Parser;
use serde::{Deserialize, Serialize};
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::{path::Path, str::FromStr};

/// global settings for web server
#[derive(Debug, Parser, Serialize, Deserialize, Clone)]
pub struct Settings {
    #[clap(flatten)]
    #[serde(flatten)]

    pub server: ServerConfig,
    #[clap(flatten)]
    #[serde(flatten)]

    pub db: DbConfig,
    #[clap(flatten)]
    #[serde(flatten)]
    pub engine: Args,
}

/// config for connecting to sqlite db
#[derive(Debug, Parser, Serialize, Deserialize, Clone)]
pub struct DbConfig {
    #[clap(long="sqlite-db", default_value="/app/data/sqlite/app.db")]
    pub file: String,
}
impl DbConfig {
    pub async fn create_pool(&self) -> sqlx::Result<SqlitePool> {
        // create db file if !exists
        if let Some(parent_dir) = Path::new(&self.file).parent() {
            std::fs::create_dir_all(parent_dir).expect("failed to create sqlite dir");
        }
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

#[derive(Debug, Parser, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[clap(long, default_value = "0.0.0.0")]
    pub host: String,
    #[clap(long="web-port", default_value_t = 2049)]
    pub port: u16,
}
impl ServerConfig {
    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
