use crate::server::AppState;
use anyhow::Result;
use axum::extract::{Path, Query, State};
use konteksto_engine::config::Lang;

/// GET `/{lang}/game/{id}/play?word={word}`
pub async fn play(word: Query<String>, path: Path<(Lang, u32)>, state: State<AppState>) -> Result<()> {
    let mut app_state = state.0;
    let (lang, game_id) = path.0;

    app_state.maybe_update_internals(lang, game_id).await;

    // insert guess into db
    let score = app_state.play(&word).await.unwrap();
    app_state.sqlite.register_guess(&word, score).await?;

    // update recommender engine
    app_state.update_suggestion(word.0).await;

    Ok(())
}

/// GET `/{lang}/game/{id}/suggest`
pub async fn suggest(path: Path<(Lang, u32)>, state: State<AppState>) -> String {
    let mut app_state = state.0;
    let (lang, game_id) = path.0;
    app_state.maybe_update_internals(lang, game_id).await;

    app_state.suggestion.clone()
}



