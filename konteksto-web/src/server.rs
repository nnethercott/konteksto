use crate::{
    config::Settings,
    routes::get_routes,
    state::{AppState, InnerState},
};
use anyhow::Result;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

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

        let state = InnerState::from_config(&self.config).await?;
        let state = AppState(Arc::new(state));

        if let Err(e) = axum::serve(listener, self.app.with_state(state)).await {
            todo!()
        }

        Ok(())
    }
}
