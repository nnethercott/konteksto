use crate::state::AppState;
use axum::{response::Redirect, routing::{get, post}, Router};
use back::{play, suggest};
use front::main;

pub mod back;
pub mod front;

/// available routes for web app
pub fn get_routes() -> Router<AppState> {
    let backend_routes = Router::new()
        .route("/play", post(play))
        .route("/suggest", post(suggest));

    let frontend_routes = Router::new().route("/", get(main));

    Router::new()
        .route("/", get(|| async { Redirect::permanent("/en/game/42/") }))
        .nest("/{lang}/game/{id}/", frontend_routes)
        .nest("/api/{lang}/game/{id}/", backend_routes)
}
