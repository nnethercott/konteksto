use anyhow::Result;
use engine::qdrant::{Client, Entry};
use qdrant_client::qdrant::{Query, QueryPointsBuilder, Sample, SearchPointsBuilder};

#[tokio::main]
async fn main() -> Result<()> {
    let client = Client::from_grpc("http://localhost:6334")?;

    if !client.collection_exists("en").await? {
        const FILE: &'static str = "../data/embeds/en-embeds.txt";
        client.create_from_dump(FILE, "en").await?;
    }

    let mut query = client.get_random_query("en").await?;

    let res = client
        .search_points(SearchPointsBuilder::new("en", query, 1).with_payload(true))
        .await;

    println!("{:?}", res);

    Ok(())
}
