use crate::{config::Settings, db::SqliteClient};
use anyhow::Result;
use konteksto_engine::{
    Solver,
    clients::Contexto,
    config::Lang,
    solver::{LinearSolver, Step},
};
use std::sync::Mutex as StdMutex;
use std::{ops::Deref, sync::Arc};
use tokio::sync::Mutex as TokioMutex;

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
    pub engine: TokioMutex<Solver>,
    pub contexto_api: TokioMutex<Contexto>,
    pub suggestion: TokioMutex<String>,
}
impl InnerState {
    pub async fn from_config(config: &Settings) -> Result<Self> {
        let pool = config.db.create_pool().await?;

        // spin up the application logic; build qdrant indexes if they don't already
        let engine = konteksto_engine::setup(config.engine.clone()).await?;
        let contexto_api = engine.contexto.clone();

        Ok(Self {
            sqlite: SqliteClient::new(pool),
            engine: TokioMutex::new(engine),
            contexto_api: TokioMutex::new(contexto_api),
            suggestion: TokioMutex::new("random".into()),
        })
    }

    pub async fn maybe_update_internals(&self, lang: Lang, game_id: u32) {
        let k = &mut self.engine.lock().await;

        if lang != k.contexto.lang || game_id != k.contexto.game_id {
            // state
            println!("updating to {:?} and {}", &lang, game_id);
            k.qdrant.collection = lang.to_string();
            k.contexto = Contexto::new(lang, game_id);
            k.reset();

            // our own connection
            *self.contexto_api.lock().await = Contexto::new(lang, game_id);
        }
    }

    pub async fn play(&self, word: &str) -> Result<u32> {
        self.contexto_api
            .lock()
            .await
            .play(&word)
            .await
            .map_err(|_| anyhow::anyhow!("failed to play turn with contexto"))
    }

    pub async fn notify_solver(&self, word: String) -> Result<()> {
        let solver = &mut self.engine.lock().await;

        solver.ban_words(vec![word.clone()]);

        let embed = match solver.qdrant.get_embedding(word.clone()).await {
            Some(v) => v,
            None => anyhow::bail!("word not found"),
        };

        // NOTE: returns previous guess if it was worse than our best
        // idea: sample near
        let suggestion = match solver.next_step(embed).await? {
            Step::Next(attempt, vector) => solver.qdrant.get_word(vector).await?,
            Step::Done => word,
            _ => unreachable!(),
        };
        println!("suggestion: {}", &suggestion);

        {
            let mut s = self.suggestion.lock().await;
            *s = suggestion;
        }

        Ok(())
    }
}
