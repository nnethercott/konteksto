use anyhow::Result;
use axum::extract::{Path, Query, State};
use konteksto_engine::config::Lang;
use serde::Deserialize;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PlayQuery {
    pub word: String,
}

/// GET `api/{lang}/game/{id}/play?word={word}`
pub async fn play(
    Query(PlayQuery { word }): Query<PlayQuery>,
    Path((lang, game_id)): Path<(Lang, u32)>,
    state: State<AppState>,
) {
    println!("inside");
    let mut app_state = state.0;

    app_state.maybe_update_internals(lang, game_id).await;

    // insert guess into db
    let score = app_state.play(&word).await.unwrap();
    println!("score: {}", score);

    // app_state.sqlite.register_guess(&word, score).await.unwrap();

    // update recommender engine
    app_state.notify_solver(word).await;
}

/// GET `api/{lang}/game/{id}/suggest`
pub async fn suggest(path: Path<(Lang, u32)>, state: State<AppState>) -> String {
    let mut app_state = state.0;
    let (lang, game_id) = path.0;
    app_state.maybe_update_internals(lang, game_id).await;

    app_state.suggestion.lock().await.clone()
}
