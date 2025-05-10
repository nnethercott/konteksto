use serde::Deserialize;
use crate::config::Lang;

const CONTEXTO_API_URL: &'static str = "https://api.contexto.me/machado";

/// json schema of the GET /word endpoint
#[allow(dead_code)]
#[derive(Deserialize)]
struct ContextoPayload {
    pub distance: u32,
    pub lemma: String,
    pub word: String,
}

/// A struct for making api calls to contexto
#[derive(Clone)]
pub struct Contexto {
    client: reqwest::Client,
    pub game_id: u32,
    pub lang: Lang,
}
impl Contexto {
    pub fn new(lang: Lang, game_id: u32) -> Self {
        let client = reqwest::Client::new();
        Self {
            lang,
            game_id,
            client,
        }
    }

    pub async fn play(&self, word: &str) -> reqwest::Result<u32> {
        let payload = self
            .client
            .get(format!(
                "{}/{}/game/{}/{word}",
                CONTEXTO_API_URL,
                self.lang.to_string(),
                self.game_id
            ))
            .send()
            .await?
            .error_for_status()?
            .json::<ContextoPayload>()
            .await?;

        Ok(payload.distance)
    }
}
