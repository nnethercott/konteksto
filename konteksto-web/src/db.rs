use sqlx::prelude::FromRow;

#[derive(FromRow)]
pub struct Guess {
    word: String,
    score: u32,
}

// ome helpers go here
