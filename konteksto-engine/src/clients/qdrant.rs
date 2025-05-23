use anyhow::Result;
use qdrant_client::{
    Payload, Qdrant,
    qdrant::{
        Condition, CountPointsBuilder, CreateCollectionBuilder, Datatype,
        Distance, Filter, PointStruct, Query, QueryPointsBuilder, QueryResponse,
        RecommendInputBuilder, Sample, ScoredPoint, ScrollPointsBuilder, UpsertPointsBuilder,
        VectorParamsBuilder, vectors_output::VectorsOptions,
    },
};
use serde::Deserialize;
use serde_json::json;
use std::ops::Deref;
use uuid::Uuid;

use crate::Args;

/// jsonl schema from python dump
#[derive(Deserialize, Debug)]
pub struct Entry {
    pub word: String,
    pub embedding: Vec<f32>,
}
impl Entry {
    pub fn read_from_dump(file: &str) -> Result<Vec<Self>> {
        let entries = std::fs::read_to_string(file)?
            .lines()
            .into_iter()
            .map(|l| serde_json::from_str(l))
            .collect::<Result<_, _>>()?;

        Ok(entries)
    }
}

impl Into<PointStruct> for Entry {
    fn into(self) -> PointStruct {
        let payload: Payload = json!({"word": self.word}).try_into().unwrap();
        PointStruct::new(Uuid::new_v4().to_string(), self.embedding, payload)
    }
}

/// a wrapper around a `qdrant::Client` exposing convenience methods
pub struct Qdrnt {
    inner: Qdrant,
    pub collection: String,
}

impl Deref for Qdrnt {
    type Target = Qdrant;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// util for points storing a single embedding
pub fn get_inner_vec(v: &ScoredPoint) -> Option<Vec<f32>> {
    v.vectors.as_ref().map(|inner| match inner.vectors_options {
        Some(VectorsOptions::Vector(ref w)) => w.data.clone(),
        _ => unreachable!("we're not using sparse vecs!"),
    })
}

pub fn get_neighbors_from_response(response: &QueryResponse) -> Vec<Entry> {
    let words = response
        .result
        .iter()
        .filter_map(|v| v.payload.get("word"))
        .map(|i| i.as_str().unwrap());

    let embeds = response.result.iter().filter_map(get_inner_vec);

    words
        .zip(embeds)
        .map(|(w, e)| Entry {
            word: w.to_string(),
            embedding: e,
        })
        .collect()
}

impl Qdrnt {
    pub fn new(config: Args) -> Result<Self> {
        let grpc_port = format!("http://{}:{}", &config.grpc_host, &config.grpc_port);
        let inner = Qdrant::from_url(&grpc_port).build()?;

        let collection = config.lang.to_string();

        Ok(Self { inner, collection })
    }
    pub async fn create_from_dump(&self, file: &str, collection: Option<&str>) -> Result<()> {
        let collection = collection.unwrap_or(&self.collection);
        let entries = Entry::read_from_dump(file)?;

        // create collection
        self.create_collection(
            CreateCollectionBuilder::new(collection).vectors_config(
                VectorParamsBuilder::new(entries[0].embedding.len() as u64, Distance::Cosine)
                    .datatype(Datatype::Float32),
            ),
        )
        .await?;

        // upload entries
        let points: Vec<PointStruct> = entries.into_iter().map(Into::into).collect();
        self.upsert_points(UpsertPointsBuilder::new(&self.collection, points))
            .await?;

        Ok(())
    }

    pub async fn get_random_vecs(&self, how_many: u64) -> Result<Vec<Vec<f32>>> {
        let res = self
            .query(
                QueryPointsBuilder::new(&self.collection)
                    .query(Query::new_sample(Sample::Random))
                    .with_vectors(true)
                    .limit(how_many),
            )
            .await?;

        let vectors: Vec<Vec<f32>> = res.result.iter().filter_map(|v| get_inner_vec(v)).collect();
        Ok(vectors)
    }

    pub async fn get_embedding(&self, word: String) -> Option<Vec<f32>> {
        let response = self
            .scroll(
                ScrollPointsBuilder::new(&self.collection)
                    .filter(Filter::must([Condition::matches("word", word)]))
                    .limit(1)
                    .with_vectors(true),
            )
            .await
            .ok()?;

        if response.result.len() == 0 {
            return None;
        }
        response.result[0]
            .vectors
            .as_ref()
            .map(|inner| match inner.vectors_options {
                Some(VectorsOptions::Vector(ref w)) => w.data.clone(),
                _ => unreachable!("we're not using sparse vecs!"),
            })
    }

    pub async fn get_word(&self, embedding: Vec<f32>) -> Result<String> {
        let response = self
            .query(
                QueryPointsBuilder::new(&self.collection)
                    .query(Query::new_nearest(embedding))
                    .with_payload(true)
                    .limit(1),
            )
            .await?;

        let word = response.result[0]
            .payload
            .get("word")
            .unwrap()
            .as_str()
            .unwrap()
            .to_owned();

        Ok(word)
    }

    pub async fn count_points(&self) -> Option<u64> {
        let response = self
            .count(CountPointsBuilder::new(&self.collection))
            .await
            .ok()?;

        response.result.map(|res| res.count)
    }

    pub async fn context_search(&self, positive_context: Vec<Vec<f32>>, n: u64) -> Result<Vec<Entry>>{

        let mut context = RecommendInputBuilder::default();
        for example in positive_context{
            context = context.add_positive(example);
        }

        let response = self.query(
            QueryPointsBuilder::new("{collection_name}")
                .query(context.build())
                .limit(n),
        ).await?;

        Ok(get_neighbors_from_response(&response))
    }
}
