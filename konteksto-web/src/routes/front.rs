use axum::{Router, routing::get};
use maud::{Markup, html};
use crate::server::AppState;

pub async fn hello() -> Markup {
    html! {
        head{
            script src="https://unpkg.com/htmx.org@2.0.4" {}
            link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@1.0.4/css/bulma.min.css" {}
        }
        body {
            h1 .title {"hello, world"}

            button .button .is-link
                hx-get="/nate"
                hx-target="body"
                {
                    "Click Me!"
                }
        }
    }
}

pub async fn another_route() -> Markup {
    html! {
        p{"nate was here"}
    }
}
