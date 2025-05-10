pub mod clients;
pub mod config;
pub mod errors;
pub mod solver;

use std::path::Path;

pub use clients::Qdrnt;
pub use config::Args;
use config::Lang;
pub use solver::Solver;

pub async fn setup(config: Args) -> crate::errors::Result<Solver> {
    let collection = &config.game_config.lang.to_string();

    // establish connection to vector db
    let client = Qdrnt::new(config.qdrant_config.clone())?;

    // build qdrant collections for each supported language
    for lang in vec![Lang::En, Lang::Pt, Lang::Es] {
        let lang = lang.to_string();
        let file = format!("./data/embeds/{}-embeds.txt", &lang);

        if Path::new(&file).exists() {
            if !client.collection_exists(&lang).await? {
                println!("building qdrant index for {}", &lang);
                client
                    .create_from_dump(&file, Some(&lang))
                    .await?;
            }
        }else{
            println!("WARN: embeddings for lang '{}' not found", &lang);
        }
    }

    Ok(Solver::new(config, client))
}
