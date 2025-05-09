use crate::config::OptimizerConfig;
use crate::errors::Result;
use crate::{
    clients::{Contexto, Qdrnt},
    config::Args,
};
use async_trait::async_trait;
use futures::future::join_all;
use ndarray::{Array1, Array2, Axis};
use qdrant_client::config;
use qdrant_client::qdrant::{Condition, Filter};

#[derive(PartialEq, PartialOrd)]
pub enum Step<T> {
    Done,
    Bailed((String, u32)),
    Next(T),
}

#[async_trait]
pub trait LinearSolver {
    type Target;

    async fn next_step(&mut self, prev: Self::Target) -> Result<Step<Self::Target>>;
    fn current_best(&self) -> (String, u32);
    fn reset(&mut self);
}

pub async fn solve<S>(seed: S::Target, solver: &mut S) -> Step<S::Target>
where
    S: LinearSolver,
{
    let mut prev = seed;

    loop {
        match solver.next_step(prev).await.unwrap() {
            Step::Next(next) => prev = next,
            other => return other, // Done or Bailed
        }
    }
    Step::Bailed(solver.current_best())
}

pub async fn solve_with_restarts<S>(
    solver: &mut S,
    seeds: Vec<S::Target>,
    settings: &OptimizerConfig,
) -> (String, u32)
where
    S: LinearSolver,
    <S as LinearSolver>::Target: PartialEq,
{
    let mut sols = vec![];

    for seed in seeds {
        println!("\nnew seed");
        if solve(seed, solver).await == Step::Done {
            return solver.current_best();
        }

        sols.push(solver.current_best());
        solver.reset();
    }

    let best = sols.into_iter().min_by_key(|(_, k)| *k).unwrap();
    best
}

struct SolverState {
    iter: usize,
    grad: Array1<f32>,
    best: (String, u32),
    blacklist: Vec<String>,
    settings: OptimizerConfig,
}

impl SolverState {
    fn from_config(settings: OptimizerConfig) -> Self {
        Self {
            iter: 0,
            grad: Array1::zeros(1),
            best: ("init".to_string(), 20000),
            blacklist: vec![],
            settings,
        }
    }
}

/// A struct implementing logic to solve Contexto
pub struct Solver {
    state: SolverState,
    pub qdrant: Qdrnt,
    pub contexto: Contexto,
}

impl Solver {
    pub fn new(config: Args, qdrant: Qdrnt) -> Self {
        let contexto = Contexto::new(config.game_config.lang, config.game_config.game_id);
        let state = SolverState::from_config(config.optimizer_config);

        Self {
            qdrant,
            contexto,
            state,
        }
    }

    /// send request to contexto api for current game
    async fn play(&self, word: &str) -> Result<u32> {
        Ok(self.contexto.play(word).await?)
    }

    /// radomly generate seed at game start
    pub async fn generate_seed(&self, from: u64) -> Result<Vec<f32>> {
        let vecs = &self.qdrant.get_random_vecs(from).await?;

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

    async fn next_step(&mut self, query: Self::Target) -> Result<Step<Self::Target>> {
        if self.state.iter >= self.state.settings.max_iters {
            return Ok(Step::Bailed(self.current_best()));
        }
        self.state.iter += 1;

        let dim = query.len();

        // explore nearby samples with blacklist
        let conds: Vec<Condition> = self
            .state
            .blacklist
            .iter()
            .map(|w| Condition::matches("word", w.clone()))
            .collect();

        let neighbors = self
            .qdrant
            .explore_with_conds(&query, Filter::must_not(conds), 3)
            .await?;

        // prevent from exploring those words next iteration (tabu-like)
        self.state
            .blacklist
            .extend(neighbors.iter().map(|e| e.word.clone()));

        // try each guess with contexto api
        let ranks = join_all(neighbors.iter().map(|entry| self.play(&entry.word))).await;

        let mut scored_neighbors: Vec<_> = neighbors
            .into_iter()
            .zip(ranks)
            .filter_map(|(entry, result)| {
                result.ok().map(|rank| (entry.word, entry.embedding, rank))
            })
            .collect();

        if scored_neighbors.is_empty() {
            return Err(anyhow::anyhow!("No neighbors found").into());
        }

        // find optimal neighbor
        scored_neighbors.sort_by_key(|(_, _, rank)| *rank);
        let (best_word, best_embedding, best_rank) = &scored_neighbors[0];
        println!(
            r#"guess: "{best_word}" rank: {best_rank}, best: {:?}"#,
            self.state.best
        );

        // if current score is worse (within a tolerance) don't explore
        if *best_rank < self.state.best.1 {
            self.state.best = (best_word.to_owned(), *best_rank);
        } else if *best_rank > self.state.settings.margin {
            return Ok(Step::Next(query));
        };

        // early stopping
        if *best_rank == 0 {
            return Ok(Step::Done);
        }

        // hill climbing with momentum
        let origin = Array1::from_shape_vec(dim, query.clone())?;
        let chosen = Array1::from_shape_vec(dim, best_embedding.clone())?;

        let dir = &chosen - &origin;
        let g = dir * (self.state.best.1 as f32) / (*best_rank as f32);

        let beta = self.state.settings.beta;
        self.state.grad = beta * &self.state.grad + (1.0 - beta) * g;
        let next_query = origin + &self.state.grad;

        Ok(Step::Next(next_query.to_vec()))
    }

    fn current_best(&self) -> (String, u32) {
        let best = &self.state.best;
        (best.0.clone(), best.1.clone())
    }

    fn reset(&mut self) {
        self.state.best = ("".to_string(), u32::MAX);
        self.state.blacklist.clear();
        self.state.grad = Array1::zeros(1);
        self.state.iter = 0;
    }
}
