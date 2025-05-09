use axum::{Router, routing::get};
use maud::{Markup, html};

use crate::server::AppState;

pub mod play;
pub mod suggest;

async fn hello() -> Markup {
    html! {
        head{
            script src="https://unpkg.com/htmx.org@2.0.4" {}
        }
        body {
            h2{"hello, world"}

            button
                hx-get="/nate"
                hx-target="body"
                {
                    "Click Me!"
                }
        }
    }
}

async fn another_route() -> Markup {
    html! {
        p{"nate was here"}
    }
}

pub fn get_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(hello))
        .route("/nate", get(another_route))
}
