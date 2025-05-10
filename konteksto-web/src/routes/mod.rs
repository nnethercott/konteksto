use crate::server::AppState;
use axum::{Router, routing::get};
use back::{play, suggest};
use front::{another_route, hello};

pub mod back;
pub mod front;

/// available routes for web app
pub fn get_routes() -> Router<AppState> {
    // let backend = Router::new()
    //     .route("/play", get(play))
    //     .route("/suggest", suggest);

    let frontend = Router::new()
        .route("/", get(hello))
        .route("/nate", get(another_route));

    // TODO: add a redirect from / to /en/game/42/play
    Router::new()
        .merge(frontend)
        // .nest("/{lang}/game/{id}/play", backend)
}
