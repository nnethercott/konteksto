use clap::Parser;
use konteksto_engine::Args;
use konteksto_web::{config::{DbConfig, ServerConfig, Settings}, server::App};

#[tokio::main(flavor="current_thread")]
async fn main() -> anyhow::Result<()>{
    let args = Args::parse();

    let config = Settings{
        server: ServerConfig{host: "0.0.0.0".into(), port: 8000},
        db: DbConfig{file: "data/sqlite/app.db".into()},
        engine: args
    };

    App::new(config).run().await?;
    Ok(())
}
