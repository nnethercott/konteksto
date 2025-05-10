use std::str::FromStr;

use clap::Parser;
use serde::{self, Deserialize, Serialize};

const QDRANT_SERVICE_GRPC_PORT: &str = "QDRANT__SERVICE__GRPC_PORT";

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum Lang {
    #[default]
    En,
    Pt,
    Es,
}
impl ToString for Lang {
    fn to_string(&self) -> String {
        match self {
            Lang::En => "en".to_string(),
            Lang::Pt => "pt-br".to_string(),
            Lang::Es => "es".to_string(),
        }
    }
}

impl FromStr for Lang {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "en" => Ok(Lang::En),
            "pt-br" | "pt" => Ok(Lang::Pt),
            "es" => Ok(Lang::Es),
            _ => Err(format!("Invalid language: {}", input)),
        }
    }
}

#[derive(Parser, Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Args {
    #[serde(flatten)]
    #[clap(flatten)]
    pub game_config: GameConfig,

    #[serde(flatten)]
    #[clap(flatten)]
    pub qdrant_config: QdrntConfig,

    #[serde(flatten)]
    #[clap(flatten)]
    pub optimizer_config: OptimizerConfig,
}

#[derive(Parser, Serialize, Deserialize, Debug, Copy, Clone)]
pub struct GameConfig {
    /// id of game as seen on screen
    #[clap(long, default_value_t = 42)]
    pub game_id: u32,

    /// language to play in; available langs are: 'en', 'pt-br', and 'es'
    #[serde(default)]
    #[clap(short, long, default_value = "en")]
    pub lang: Lang,
}

#[derive(Parser, Serialize, Deserialize, Debug, Copy, Clone)]
pub struct QdrntConfig {
    /// grpc port where qdrant db is running on
    #[clap(long, env = QDRANT_SERVICE_GRPC_PORT, default_value_t=6334)]
    pub grpc_port: u32,

    #[clap(long, default_value = "en")]
    pub collection: Lang,
}

#[derive(Parser, Default, Serialize, Deserialize, Debug, Copy, Clone)]
pub struct OptimizerConfig {
    /// number of times to randomly initialize search algorithm
    #[clap(long, default_value_t = 1)]
    pub max_retries: usize,

    /// max number of iterations per solution attempt
    #[clap(long, default_value_t = 100)]
    pub max_iters: usize,

    /// decay rate in momemntum update
    #[clap(long, default_value_t = 0.5)]
    pub beta: f32,

    /// value under which "free mobility" is possible
    #[clap(long, default_value_t = 200)]
    pub margin: u32,
}
