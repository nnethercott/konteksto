use std::cmp::Ordering;

use crate::{
    contexto::{self, Lang},
    qdrant::{self, get_entries_from_response, get_inner_vec},
};
use anyhow::Result;
use async_trait::async_trait;
use ndarray::{Array1, Array2, Axis, arr2};
use qdrant_client::qdrant::{Query, QueryPointsBuilder, SearchPointsBuilder};

pub enum Turn<T> {
    Done,
    Bailed(String),
    Next(T),
}

#[async_trait]
pub trait LinearSolver {
    type Target;

    async fn next_step(&self, prev: Self::Target) -> Result<Turn<Self::Target>>;
}

pub async fn solve<S>(seed: S::Target, solver: S)
where
    S: LinearSolver,
{
    let mut prev = seed;
    for _ in 0..100 {
        prev = match solver.next_step(prev).await.unwrap() {
            Turn::Next(n) => n,
            _ => break,
        };
    }
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

// something with history ?
struct SolverState; 

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

// TODO: try secant method

#[async_trait]
impl LinearSolver for Solver {
    type Target = Vec<f32>;

    async fn next_step(&self, query: Self::Target) -> Result<Turn<Self::Target>> {
        let dim = query.len();

        let response = self
            .qdrant
            .query(
                QueryPointsBuilder::new(&self.config.collection_name)
                    .query(Query::new_nearest(query.clone()))
                    .with_payload(true)
                    .with_vectors(true)
                    .limit(2),
            )
            .await?;

        let mut similar = get_entries_from_response(&response);
        let e1 = similar.remove(0);
        let e2 = similar.remove(0);

        // get rank from contexto api
        let (r1, r2) = tokio::join!(self.play(&e1.word), self.play(&e2.word));

        // apply numerical methods
        let r1 = r1.unwrap();
        let r2 = r2.unwrap();

        let u = Array1::from_shape_vec(dim, e1.embedding)?;
        let v = Array1::from_shape_vec(dim, e2.embedding)?;
        let origin = Array1::from_shape_vec(dim, query)?;

        let (chosen, rank) = match r1.cmp(&r2) {
            Ordering::Less => (u, r1),
            _ => (v, r2),
        };

        println!("rank: {rank}");

        if rank == 0 {
            return Ok(Turn::Done);
        }

        // step
        let mut dir = &chosen - &origin;
        let norm: f32 = dir.mapv(|x| x * x).sum().sqrt();
        let alpha = norm * (dim as f32).sqrt();
        let next_query = &chosen + &dir.mapv(|x| 0.1 * x * (rank as f32) / (alpha + 1e-05));

        Ok(Turn::Next(next_query.to_vec()))
    }
}
