use anyhow::Result;
use axum::Router;
use konteksto_engine::{
    self, clients::Contexto, config::Lang, solver::{LinearSolver, Step}, Solver
};
use std::sync::Arc;
use tokio::{net::TcpListener, sync::Mutex};
use tower_http::services::ServeDir;

use crate::{config::Settings, db::SqliteClient, routes::get_routes};

/// Internal state of web server handling all game logic
#[derive(Clone)]
pub struct AppState {
    pub sqlite: Arc<SqliteClient>,
    pub engine: Arc<Mutex<Solver>>,
    pub contexto_api: Arc<Contexto>,
    pub suggestion: String,
}

impl AppState {
    pub async fn from_config(config: &Settings) -> Result<Self> {
        let pool = config.db.create_pool().await?;

        // spin up the application logic; build qdrant indexes if they don't already
        let engine = konteksto_engine::setup(config.engine.clone()).await?;
        let contexto_api = engine.contexto.clone();

        Ok(Self {
            sqlite: Arc::new(SqliteClient::new(pool)),
            engine: Arc::new(Mutex::new(engine)),
            contexto_api: Arc::new(contexto_api),
            suggestion: "random".into(),
        })
    }

    pub async fn maybe_update_internals(&self, lang: Lang, game_id: u32) {
        let k = &mut self.engine.lock().await;

        if lang != k.contexto.lang || game_id != k.contexto.game_id {
            k.qdrant.collection = lang.to_string();
            k.contexto = Contexto::new(lang, game_id);
            k.reset();
        }
    }

    pub async fn play(&self, word: &str) -> Result<u32> {
        self.contexto_api
            .play(&word)
            .await
            .map_err(|_| anyhow::anyhow!("failed to play turn with contexto"))
    }

    pub async fn update_suggestion(&mut self, word: String) -> Result<()> {
        let solver = &mut self.engine.lock().await;

        solver.ban_words(vec![word.clone()]);

        let embed = match solver.qdrant.get_embedding(word.clone()).await {
            Some(v) => v,
            None => anyhow::bail!("word not found"),
        };

        // next candidate word
        self.suggestion = match solver.next_step(embed).await?{
            Step::Next(_, vector) => solver.qdrant.get_word(vector).await?,
            _ => word,
        };

        Ok(())
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
