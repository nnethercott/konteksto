use axum::{extract::Path, response::IntoResponse, routing::get, Router};
use konteksto_engine::config::Lang;
use maud::{DOCTYPE, Markup, Render, html};
use sqlx::query::Query;
use crate::db::Guess;

pub async fn main(Path((lang, game_id)): Path<(Lang, u32)>) -> Markup {
    let guesses = vec![Guess{word: "nate".into(), score: 20}];
    let home = Home{lang, game_id, guesses};
    home.render()
}

pub struct Home {
    game_id: u32,
    lang: Lang,
    guesses: Vec<Guess>,
}

impl Render for Home {
    fn render(&self) -> Markup {
        html! {
            (DOCTYPE)
            head {
                title { "Kontektso" }
                meta name="viewport" content="width=device-width, initial-scale=1" {}
                script src="https://unpkg.com/htmx.org@2.0.4" {}
                link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@1.0.4/css/bulma.min.css" {}
                link rel="stylesheet" href="/public/css/app.css" {}
            }
            body {
                main .section {
                    .container.has-text-centered {
                        h1 .title { "Kontektso" }
                        
                        // Input and button
                        div .input-container {
                            .control.is-expanded {
                                input
                                    hx-trigger="keydown[key==='Enter'&&!shiftKey]"
                                    hx-get=(format!("/api/{}/game/{}/play", self.lang.to_string(), self.game_id))
                                    type="text"
                                    class="input"
                                    placeholder="type a word"
                                    name="word";
                            }
                            .control.button-control {
                                button
                                    class="button is-link"
                                    hx-get="/foobar"
                                    hx-target="#guesses"
                                    hx-include="closest form"
                                    {
                                        "Suggest"
                                    }
                            }
                        }
                        
                        // Guesses list in a fixed-width div
                        div .guesses-container {
                            ul #guesses {
                                @for guess in self.guesses.iter() {
                                    (render_guess(guess))
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn render_guess(g: &Guess) -> Markup {
    html! {
        li .box.compact-box {
            span .word { (g.word) }
            span .score { (g.score) }
        }
    }
}
