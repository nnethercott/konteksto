use crate::{db::Attempt, errors::Result as AppResult, state::AppState};
use axum::extract::{Path, State};
use maud::{DOCTYPE, Markup, Render, html};

pub async fn main(
    Path(game_id): Path<u32>,
    State(AppState(app)): State<AppState>,
) -> AppResult<Markup> {
    app.maybe_reset(game_id).await?;

    // get attempts ordered by score
    let mut guesses = app.sqlite.all_guesses().await?;
    guesses.sort_by_key(|a| a.score);

    let home = Home { game_id, guesses };
    Ok(home.render())
}

/// home page for the app
pub struct Home {
    game_id: u32,
    guesses: Vec<Attempt>,
}

impl Render for Home {
    fn render(&self) -> Markup {
        let api_stub = format!("/api/game/{}", self.game_id);

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
                                    id="guess-input"
                                    type="text"
                                    name="word"
                                    class="input"
                                    placeholder="type a word"
                                    hx-trigger="keydown[key==='Enter'&&!shiftKey]"
                                    hx-post=(format!("{}/play", api_stub))
                                    hx-on::after-request="if(event.detail.successful) window.location.reload();";
                            }
                            .control.button-control {
                                button
                                    class="button is-link"
                                    hx-post=(format!("{}/suggest", api_stub))
                                    hx-target="#guess-input"
                                    hx-swap="outerHTML"
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

fn render_guess(g: &Attempt) -> Markup {
    html! {
        li .box.my-2.compact-box {
            span .word { (g.word) }
            span .score { (g.score + 1) }
        }
    }
}
