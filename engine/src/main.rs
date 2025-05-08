use anyhow::Result;
use engine::{
    qdrant::{Client, Entry},
    solver::{solve, Solver},
};
use qdrant_client::qdrant::{Query, QueryPointsBuilder, Sample, SearchPointsBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::from_grpc("http://localhost:6334")?;

    if !client.collection_exists("en").await? {
        const FILE: &'static str = "../data/embeds/en-embeds.txt"; // FIXME!
        client.create_from_dump(FILE, "en").await?;
    }

    let solver = Solver::new(0, "en", client);
    solve(solver.generate_seed().await?, solver).await;

    Ok(())
}
