use anyhow::Result;
use clap::Parser;
use konteksto_engine::{
    config::Args,
    solver::solve_with_restarts,
};
use konteksto_engine::setup;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let config = Args::parse();
    dbg!("{:?}", &config);

    let mut solver = setup(config.clone()).await?;

    // solve with max retries
    let max_retries = config.optimizer_config.max_retries;

    let mut seeds = vec![];
    for _ in 0..max_retries {
        seeds.push(solver.generate_seed(1).await?);
    }

    let best = solve_with_restarts(&mut solver, seeds, &config.optimizer_config).await;
    dbg!("{:?}", best);

    Ok(())
}
