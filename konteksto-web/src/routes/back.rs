use axum::Form;
use axum::extract::{Path, State};
use maud::{Markup, html};
use serde::Deserialize;

use crate::errors::Result as AppResult;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct PlayQuery {
    pub word: String,
}

/// POST `api/{lang}/game/{id}/play`
pub async fn play(
    Path(game_id): Path<u32>,
    State(AppState(app_state)): State<AppState>,
    Form(PlayQuery { word }): Form<PlayQuery>,
) -> AppResult<()> {
    app_state.maybe_reset(game_id).await?;

    let score = app_state.play(&word).await?;
    println!("score: {}", score);

    app_state.sqlite.register_guess(&word, score).await?;

    // update recommender engine
    app_state.notify_solver(word).await?;

    Ok(())
}

/// POST `api/{lang}/game/{id}/suggest`
pub async fn suggest(
    Path(game_id): Path<u32>,
    State(AppState(app_state)): State<AppState>,
) -> AppResult<Markup> {
    app_state.maybe_reset(game_id).await?;

    let suggestion = app_state.suggestion.lock().await.clone();
    println!("{}", suggestion);

    // swaps outer html
    Ok(html! {
        input
            id="guess-input"
            type="text"
            name="word"
            class="input"
            placeholder="type a word"
            value=(suggestion)
            hx-trigger="keydown[key==='Enter'&&!shiftKey]"
            hx-post=(format!("/api/game/{}/play", game_id))
            hx-on::after-request="if(event.detail.successful) window.location.reload();";
    })
}
