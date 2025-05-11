use crate::{config::Settings, db::SqliteClient};
use anyhow::Result;
use konteksto_engine::{
    Solver,
    clients::Contexto,
    solver::{LinearSolver, Step},
};
use tracing::info;
use std::{ops::Deref, sync::Arc};
use tokio::sync::Mutex;

/// Internal state of web server handling all game logic
#[derive(Clone)]
pub struct AppState(pub Arc<InnerState>);
impl Deref for AppState {
    type Target = InnerState;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct InnerState {
    pub sqlite: SqliteClient,
    pub engine: Mutex<Solver>,
    pub contexto_api: Mutex<Contexto>,
    pub suggestion: Mutex<String>,
}
impl InnerState {
    pub async fn from_config(config: &Settings) -> Result<Self> {
        let pool = config.db.create_pool().await?;

        // spin up the application logic; build qdrant indexes if they don't already
        let engine = konteksto_engine::setup(config.engine.clone()).await?;
        let contexto_api = engine.contexto.clone();

        // random seed for suggestion
        let random_vec = engine.generate_seed(1).await?;
        let random_word = engine.qdrant.get_word(random_vec).await?;
        info!("random: {}", &random_word);

        Ok(Self {
            sqlite: SqliteClient::new(pool),
            engine: Mutex::new(engine),
            contexto_api: Mutex::new(contexto_api),
            suggestion: Mutex::new(random_word),
        })
    }

    /// manual reset
    pub async fn maybe_reset(&self, game_id: u32) -> Result<()> {
        let engine = &mut self.engine.lock().await;

        if game_id != engine.contexto.game_id {
            info!("updating to game {}", game_id);

            // solver
            engine.reset();

            // api 
            let contexto = Contexto::new(engine.contexto.lang, game_id);
            engine.contexto = contexto.clone();
            *self.contexto_api.lock().await = contexto;

            // db
            self.sqlite.delete_all_guesses().await?;

            // & new random suggestion
            {
                let random_vec = engine.generate_seed(1).await?;
                let mut s = self.suggestion.lock().await;
                *s = engine.qdrant.get_word(random_vec).await?;
            }
        }
        Ok(())
    }

    pub async fn play(&self, word: &str) -> Result<u32> {
        self.contexto_api
            .lock()
            .await
            .play(&word)
            .await
            .map_err(|e| anyhow::anyhow!("failed to play turn with contexto\n{:?}", e))
    }

    /// Manually step the engine and generate a new suggestion
    /// variant of algo in konteksto-engine/solver.rs
    pub async fn notify_solver(&self, word: String) -> Result<()> {
        let solver = &mut self.engine.lock().await;

        let embed = match solver.qdrant.get_embedding(word).await {
            Some(v) => v,
            None => anyhow::bail!("word not found"),
        };

        let prev_best = solver.current_best();
        let suggestion = match solver.next_step(embed).await? {
            Step::Next(attempt, next_query) => {
                info!("best: {:?}, attempt: {:?}", &prev_best, &attempt);

                // if no change re-use a word near the local min
                let best_query = if attempt.1 > prev_best.1 {
                    solver.qdrant.get_embedding(prev_best.0).await.unwrap()
                } else {
                    next_query
                };

                let mut nearest_neighbors = solver.query_unseen(best_query, 1).await?;
                nearest_neighbors.remove(0).word
            }
            Step::Done => solver.current_best().0,
            _ => unreachable!(),
        };

        {
            let mut s = self.suggestion.lock().await;
            *s = suggestion;
        }

        Ok(())
    }
}
