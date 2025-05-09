use anyhow::Result;
use axum::Router;
use konteksto_engine::{self, Args, Qdrnt, Solver, clients::qdrant};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::{config::Settings, routes::get_routes};

#[derive(Clone)]
pub struct AppState {
    pool: Arc<SqlitePool>,
    engine: Arc<Solver>,
}
impl AppState {
    pub async fn from_config(config: &Settings) -> Result<Self> {
        // connect to db
        let pool = config.db.create_pool().await?;
        // spin up the application logic
        let engine = konteksto_engine::setup(config.engine.clone()).await?;

        Ok(Self {
            pool: Arc::new(pool),
            engine: Arc::new(engine),
        })
    }
}

pub struct App {
    app: Router<AppState>,
    config: Settings,
}
impl App {
    pub fn new(config: Settings) -> Self {
        // inject htmx.min.js
        let app = Router::new()
            .nest_service("/public", ServeDir::new("./konteksto-web/public"))
            .merge(get_routes());

        Self { app, config }
    }
    pub async fn run(self) -> anyhow::Result<()> {
        let listener = TcpListener::bind(self.config.server.addr()).await?;

        let state = AppState::from_config(&self.config).await?;

        if let Err(e) = axum::serve(listener, self.app.with_state(state)).await {
            todo!()
        }

        Ok(())
    }
}
