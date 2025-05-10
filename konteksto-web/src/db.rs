use sqlx::{SqlitePool, prelude::FromRow};

#[derive(FromRow)]
pub struct Guess {
    pub word: String,
    pub score: u32,
}

pub struct SqliteClient(SqlitePool);

impl SqliteClient {
    pub fn new(pool: SqlitePool) -> Self {
        Self(pool)
    }

    pub async fn register_guess(&self, word: &str, score: u32) -> sqlx::Result<()> {
        let _ = sqlx::query(&format!(
            r#"INSERT INTO guesses(word, score) VALUES({}, {})"#,
            &word, score
        ))
        .execute(&self.0)
        .await?;
        Ok(())
    }
}
