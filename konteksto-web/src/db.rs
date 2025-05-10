use sqlx::{SqlitePool, prelude::FromRow};

#[derive(FromRow)]
pub struct Guess {
    word: String,
    score: u32,
}

pub struct SqliteClient(SqlitePool);

impl SqliteClient {
    pub fn new(pool: SqlitePool)->Self{
        Self(pool)
    }

    pub async fn register_guess(&self, word: &str, score: u32) -> sqlx::Result<()> {
        let _ = sqlx::query!(
            r#"INSERT INTO guesses(word, score) VALUES($1, $2)"#,
            word,
            score
        )
        .execute(&self.0)
        .await?;
        Ok(())
    }
}
