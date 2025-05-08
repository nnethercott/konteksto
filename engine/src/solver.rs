use crate::{
    contexto::{self, Lang},
    qdrant::{self, get_entries_from_response, get_inner_vec},
};
use anyhow::Result;
use async_trait::async_trait;
use ndarray::{Array2, Axis, arr2};
use qdrant_client::qdrant::{Query, QueryPointsBuilder, SearchPointsBuilder};

pub enum Status<T> {
    Done,
    Bailed(String),
    Next(T),
}

#[async_trait]
pub trait LinearSolver {
    type Target;

    async fn next_step(&self, prev: Self::Target) -> Result<Status<Self::Target>>;
}

pub async fn solve<S>(seed: S::Target, solver: S)
where
    S: LinearSolver,
{
    solver.next_step(seed).await.unwrap();
}

pub struct Config {
    collection_name: String,
    lang: Lang,
}

/// A struct containing logic to solve Contexto
pub struct Solver {
    pub config: Config,
    pub qdrant: qdrant::Client,
    pub contexto: contexto::Contexto,
}

impl Solver {
    pub fn new(game_id: u32, collection_name: impl ToString, qdrant: qdrant::Client) -> Self {
        let config = Config {
            collection_name: collection_name.to_string(),
            lang: Lang::En,
        };

        Self {
            qdrant,
            config,
            contexto: contexto::Contexto::new(Lang::En, game_id),
        }
    }

    /// send request to contexto api for current game
    async fn play(&self, word: &str) -> reqwest::Result<u32> {
        self.contexto.play(word).await
    }

    /// radomly generate seed at game start
    pub async fn generate_seed(&self) -> Result<Vec<f32>> {
        let vecs = &self
            .qdrant
            .get_random_vecs(&self.config.collection_name, 32)
            .await?;

        let dim = vecs[0].len();
        let seeds = Array2::from_shape_vec(
            (vecs.len(), dim),
            vecs.into_iter()
                .flat_map(|v| v.into_iter().cloned())
                .collect(),
        )?;

        let seed = seeds
            .mean_axis(Axis(0))
            .ok_or_else(|| anyhow::anyhow!("failed to compute mean!"))?
            .to_vec();

        Ok(seed)
    }
}

#[async_trait]
impl LinearSolver for Solver {
    type Target = Vec<f32>;

    async fn next_step(&self, query: Self::Target) -> Result<Status<Self::Target>> {
        let response = self
            .qdrant
            .query(
                QueryPointsBuilder::new(&self.config.collection_name)
                    .query(Query::new_nearest(query))
                    .with_payload(true)
                    .with_vectors(true)
                    .limit(2),
            )
            .await?;

        let entries = get_entries_from_response(&response);

        // get ranks from contexto api
        let ranks = tokio::join!(self.play(&entries[0].word), self.play(&entries[1].word));
        println!("{:?}", ranks);

        Ok(Status::Next(vec![42.0]))
    }
}
