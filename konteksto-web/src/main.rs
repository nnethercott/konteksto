use clap::Parser;
use konteksto_web::{config::Settings, server::App};

#[tokio::main(flavor="current_thread")]
async fn main() -> anyhow::Result<()>{
    let settings = Settings::parse();
    dbg!("{:?}", &settings);

    App::new(settings).run().await?;
    Ok(())
}
