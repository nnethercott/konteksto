use anyhow::Result;
use qdrant_client::{
    Payload, Qdrant,
    config::QdrantConfig,
    qdrant::{
        Condition, CreateCollectionBuilder, Datatype, Distance, Filter, PointStruct, Query,
        QueryPointsBuilder, QueryResponse, Sample, ScoredPoint, ScrollPointsBuilder,
        UpsertPointsBuilder, Vector, VectorParamsBuilder, Vectors, vectors_output::VectorsOptions,
    },
};
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, ops::Deref};
use uuid::Uuid;

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
pub struct Client(Qdrant);

impl Deref for Client {
    type Target = Qdrant;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// util for points storing a single embedding
pub fn get_inner_vec(v: &ScoredPoint) -> Option<Vec<f32>> {
    v.vectors.as_ref().map(|inner| match inner.vectors_options {
        Some(VectorsOptions::Vector(ref w)) => w.data.clone(),
        _ => unreachable!("we're not using sparse vecs!"),
    })
}

pub fn get_entries_from_response(response: &QueryResponse) -> Vec<Entry> {
    let words = response
        .result
        .iter()
        .filter_map(|v| v.payload.get("word"))
        .map(|i| i.as_str().unwrap());

    let embeds = response.result.iter().take(2).filter_map(get_inner_vec);

    words
        .zip(embeds)
        .map(|(w, e)| Entry {
            word: w.to_string(),
            embedding: e,
        })
        .collect()
}

impl Client {
    pub fn from_grpc(grpc_url: &str) -> Result<Self> {
        let client = Qdrant::from_url(grpc_url).build()?;
        Ok(Self(client))
    }
    pub async fn create_from_dump(&self, file: &str, collection_name: &str) -> Result<()> {
        let entries = Entry::read_from_dump(file)?;

        // create collection
        self.create_collection(
            CreateCollectionBuilder::new(collection_name).vectors_config(
                VectorParamsBuilder::new(entries[0].embedding.len() as u64, Distance::Euclid)
                    .datatype(Datatype::Float32),
            ),
        )
        .await?;

        // upload entries
        let points: Vec<PointStruct> = entries.into_iter().map(Into::into).collect();
        self.upsert_points(UpsertPointsBuilder::new(collection_name, points))
            .await?;

        Ok(())
    }

    pub async fn get_random_vecs(
        &self,
        collection_name: &str,
        how_many: u64,
    ) -> Result<Vec<Vec<f32>>> {
        let res = self
            .query(
                QueryPointsBuilder::new(collection_name)
                    .query(Query::new_sample(Sample::Random))
                    .with_vectors(true)
                    .limit(how_many),
            )
            .await?;

        let mut vectors: Vec<Vec<f32>> =
            res.result.iter().filter_map(|v| get_inner_vec(v)).collect();
        Ok(vectors)
    }

    pub fn sample_near(other: PointStruct, k: usize) -> Result<Vec<PointStruct>> {
        todo!()
    }
}
