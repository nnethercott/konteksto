use serde::Deserialize;
use std::ops::Deref;

const CONTEXTO_API_URL: &'static str = "https://api.contexto.me/machado";

pub enum Lang {
    En,
    Pt,
    Es,
}
impl ToString for Lang {
    fn to_string(&self) -> String {
        match self {
            Lang::En => "en".to_string(),
            Lang::Pt => "pt-br".to_string(),
            Lang::Es => "es".to_string(),
        }
    }
}

#[derive(Deserialize)]
struct ContextoPayload {
    pub distance: u32,
    pub lemma: String,
    pub word: String,
}

pub struct Contexto {
    client: reqwest::Client,
    game_id: u32,
    lang: Lang,
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
