use clap::Parser;
use konteksto_web::{config::Settings, server::App};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main(flavor="current_thread")]
async fn main() -> anyhow::Result<()>{
    let settings = Settings::parse();
    dbg!("{:?}", &settings);

    // tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .with_level(true)
                .with_span_list(false), // noise
        )
        .with(EnvFilter::try_from_default_env().unwrap_or("info".into()))
        .init();

    App::new(settings).run().await?;
    Ok(())
}
