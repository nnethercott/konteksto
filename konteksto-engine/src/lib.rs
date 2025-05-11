pub mod clients;
pub mod config;
pub mod errors;
pub mod solver;
pub mod title;

use std::path::Path;

pub use clients::Qdrnt;
pub use config::Args;
pub use solver::Solver;

pub async fn setup(config: Args) -> crate::errors::Result<Solver> {
    let lang = &config.lang;
    let collection = lang.to_string();

    // establish connection to vector db
    let client = Qdrnt::new(config.clone())?;

    // build qdrant collection for lang
    let file = format!("./data/embeds/{}-embeds.txt", &collection);

    if Path::new(&file).exists() {
        if !client.collection_exists(&collection).await? {
            println!("building qdrant index for {}", &collection);
            client
                .create_from_dump(&file, Some(&collection))
                .await?;
        }
    }else{
        println!("WARN: embeddings for collection '{}' not found", &collection);
    }

    Ok(Solver::new(config, client))
}
