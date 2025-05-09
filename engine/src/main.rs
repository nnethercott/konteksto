use anyhow::Result;
use clap::Parser;
use engine::{
    clients::Qdrnt,
    config::Args,
    solver::{Solver, solve_with_restarts},
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = Args::parse();
    dbg!("{:?}", &config);

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

    // solve with restarts
    let mut solver = Solver::new(config.clone(), client);
    let max_retries = config.optimizer_config.max_retries;
    let mut seeds = vec![];
    for _ in 0..max_retries {
        seeds.push(solver.generate_seed(2).await?);
    }

    let best = solve_with_restarts(&mut solver, seeds, &config.optimizer_config).await;
    dbg!("{:?}", best);

    Ok(())
}
