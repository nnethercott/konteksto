pub mod solver;
pub mod config;
pub mod errors;
pub mod clients;

pub use clients::Qdrnt;
pub use solver::Solver;
pub use config::Args;

pub async fn setup(config: Args) -> crate::errors::Result<Solver>{
    let collection = &config.game_config.lang.to_string();

    // establish connection to vector db
    let client = Qdrnt::new(
        &format!("http://localhost:{}", &config.qdrant_config.grpc_port),
        collection,
    )?;

    // build collcetion if !exists
    if !client.collection_exists(collection).await? {
        println!("building qdrant index !");
        let file = format!("./data/embeds/{}-embeds.txt", collection);
        client.create_from_dump(&file).await?;
    }

    Ok(Solver::new(config, client))
}

