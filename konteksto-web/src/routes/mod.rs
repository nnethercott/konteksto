use axum::{Router, routing::get};
use back::{play, suggest};
use front::main;
use crate::state::AppState;

pub mod back;
pub mod front;

/// available routes for web app
pub fn get_routes() -> Router<AppState> {
    let backend_routes = Router::new()
        .route("/play", get(play))
        .route("/suggest", get(suggest));

    let frontend_routes = Router::new()
        .route("/", get(main));

    Router::new()
        .nest("/{lang}/game/{id}/", frontend_routes)
        .nest("/api/{lang}/game/{id}/", backend_routes)
}
